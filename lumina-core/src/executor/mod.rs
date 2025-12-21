mod frame;
mod call_stack;
mod walk;

use std::sync::Arc;
use rustc_hash::FxHashMap;
use mlua::Lua;
use viviscript_core::ast::{Stmt, Script};
use std::collections::HashMap;
use frame::Frame;
use call_stack::CallStack;

use crate::runtime::Ctx;
use crate::event::{OutputEvent, InputEvent};
use crate::executor::walk::{walk_stmt, NextAction, StmtEffect};
use crate::lua_glue::{self, CommandBuffer, LuaCommand};
use crate::storager::types::FrameSnapshot;

#[derive(Debug, Clone)]
pub struct Executor {
    call_stack: CallStack,
    lua: Lua,
    cmd_buffer: CommandBuffer,
    pending_choice: Option<Vec<(String, Vec<Stmt>)>>,
    pause: bool,
    label_map: FxHashMap<String, Arc<[Stmt]>>,
}

impl Executor {
    pub fn new() -> Self{
        let lua = Lua::new();
        let cmd_buffer = lua_glue::init_lua(&lua);

        Executor {
            call_stack: CallStack::default(),
            lua,
            cmd_buffer,
            pending_choice: None,
            pause: false,
            label_map: FxHashMap::default(),
        }
    }

    pub fn prepare_script(script: &mut Script) {
        log::info!("Preparing script (mutating AST)...");
        pre_narration_lines(&mut script.body);

        let mut dummy_map = FxHashMap::default();
        preprocess_block(&mut script.body, "root", &mut dummy_map);
    }

    pub fn load_script(&mut self, ctx: &mut Ctx, script: Arc<Script>) {
        log::info!("Loading script (indexing)...");

        pre_collect_characters(ctx, &script.body);

        self.label_map.clear();
        self.build_label_map_from_stmts(&script.body);
    }

    fn build_label_map_from_stmts(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::Label { id, body, .. } => {
                    self.label_map.insert(id.clone(), Arc::from(body.as_slice()));
                    self.build_label_map_from_stmts(body);
                },
                Stmt::If { branches, else_branch, id, .. } => {
                    if let Some(base_id) = id {
                        for (idx, (_, body)) in branches.iter().enumerate() {
                            let branch_id = format!("{}_b{}", base_id, idx);
                            self.label_map.insert(branch_id, Arc::from(body.as_slice()));
                            self.build_label_map_from_stmts(body);
                        }
                        if let Some(body) = else_branch {
                            let branch_id = format!("{}_else", base_id);
                            self.label_map.insert(branch_id, Arc::from(body.as_slice()));
                            self.build_label_map_from_stmts(body);
                        }
                    }
                },
                Stmt::Choice { arms, id, .. } => {
                    if let Some(base_id) = id {
                        for (idx, arm) in arms.iter().enumerate() {
                            let arm_id = format!("{}_opt{}", base_id, idx);
                            self.label_map.insert(arm_id, Arc::from(arm.body.as_slice()));
                            self.build_label_map_from_stmts(&arm.body);
                        }
                    }
                },
                _ => {}
            }
        }
    }
    
    pub fn start(&mut self, ctx: &mut Ctx, label: &str) {
        init_ctx_runtime(ctx);
        let body = self.get_block_arc(label).unwrap_or_else(|| panic!("label {} not found", label));
        self.call_stack.push(Frame::new(label.to_string(),body, 0));
    }
    pub fn feed(&mut self, ev: InputEvent) {
        match ev {
            InputEvent::ChoiceMade { index } => {
                if let Some(mut arms) = self.pending_choice.take() {
                    if index < arms.len() {
                        let (block_id, selected_body) = arms.remove(index);

                        let frame = self.call_stack.top_mut().unwrap();
                        frame.advance();

                        let return_frame = Frame::new(frame.name.clone(), frame.stmts.clone(), frame.pc);
                        self.call_stack.pop();
                        self.call_stack.push(return_frame);

                        self.call_stack.push(Frame::new(block_id, selected_body, 0));
                    }
                }
            },
            InputEvent::Exit => {
                self.call_stack.clear();
                self.pause = false;
                if let Some(frame) = self.call_stack.top_mut(){
                    frame.advance();
                }
            },
            InputEvent::Continue => {
                self.pause = false;
                if let Some(frame) = self.call_stack.top_mut(){
                    frame.advance();
                }
            }
            _ => {}
        }
    }

    pub fn snapshot(&self) -> Vec<FrameSnapshot> {
        self.call_stack.stack
            .iter().map(|f| FrameSnapshot {
            label: f.name.clone(),
            pc: f.pc,
        })
            .collect()
    }

    pub fn restore(&mut self, snap: Vec<FrameSnapshot>) {
        self.call_stack.clear();
        for fs in snap {
            if let Some(body) = self.get_block_arc(&fs.label) {
                let pc = if fs.pc > body.len() { 0 } else { fs.pc };
                let frame = Frame::new(fs.label, body, pc);
                self.call_stack.push(frame);
            } else {
                log::error!("Restore failed: Code block '{}' not found.", fs.label);
                panic!("Save file mismatch");
            }
        }
    }

    pub fn step(&mut self, ctx: &mut Ctx) -> bool {
        if self.process_lua_commands(ctx) {
            return false;
        }

        if self.pending_choice.is_some() || self.pause {
            return true;
        }
        if let Some(frame) = self.call_stack.top_mut() {
            if let Some(_) = frame.current() {
                self.exec_current(ctx);
            } else {
                self.call_stack.pop();
            }
            false
        } else {
            ctx.push(OutputEvent::End);
            false
        }
    }

    fn get_block_arc(&self, name: &str) -> Option<Arc<[Stmt]>> {
        self.label_map.get(name).cloned()
    }

    fn process_lua_commands(&mut self, _ctx: &mut Ctx) -> bool {
        let cmds = self.cmd_buffer.drain();
        if cmds.is_empty() { return false; }
        for cmd in cmds {
            match cmd {
                LuaCommand::Jump(target) => {
                    log::info!("Lua Jump -> {}", target);
                    self.perform_jump(&target);
                },
            }
        }
        true
    }

    fn perform_jump(&mut self, label: &str) {
        let body = self.get_block_arc(label)
            .unwrap_or_else(|| panic!("label {} not found", label));
        self.call_stack.clear();
        self.call_stack.push(Frame::new(label.to_string(), body, 0));
    }
    
    fn exec_current(&mut self, ctx: &mut Ctx) {
        let stmt =  {
            let frame = self.call_stack.top_mut().expect("no frame");
            frame.current().expect("no stmt").clone()
        };

        let StmtEffect { events, next} = walk_stmt(ctx, &self.lua, &stmt);
        ctx.event_queue.extend(events);

        match next {
            NextAction::Continue =>{
                if let Some(frame) = self.call_stack.top_mut(){
                    frame.advance();
                }
            },
            NextAction::WaitChoice(arms) => {
                self.pending_choice = Some(arms);
            },
            NextAction::WaitInput => {
                self.pause = true;
            }
            NextAction::Jump(label) =>{
                self.perform_jump(&label);
            },
            NextAction::Call(target) => {
                let body = self.get_block_arc(&target)
                    .unwrap_or_else(|| panic!("label {} not found", target));
                let frame = self.call_stack.top_mut().unwrap();
                let return_frame = Frame::new(frame.name.clone(),frame.stmts.clone(), frame.pc + 1);
                self.call_stack.pop();
                self.call_stack.push(return_frame);
                self.call_stack.push(Frame::new(target,body, 0));
            },
            NextAction::EnterBlock(block_id, stmts) => {
                let frame = self.call_stack.top_mut().unwrap();
                let return_frame = Frame::new(frame.name.clone(), frame.stmts.clone(), frame.pc + 1);

                self.call_stack.pop();
                self.call_stack.push(return_frame);

                self.call_stack.push(Frame::new(block_id, Arc::from(stmts.as_slice()), 0));
            }
        }
    }
}
fn init_ctx_runtime(ctx: &mut Ctx) {
    ctx.audios.insert("music".to_string(), None);
    ctx.audios.insert("sound".to_string(), None);
    ctx.audios.insert("voice".to_string(), None);
    ctx.layer_record.arrange.push("master".to_string());
    ctx.layer_record.layer.insert("master".to_string(), vec![]);
}

fn pre_collect_characters(ctx: &mut Ctx, list: &[Stmt]) {
    for stmt in list {
        match stmt {
            Stmt::CharacterDef { id, name, image_tag, voice_tag, .. } => {
                ctx.characters.insert(
                    id.clone(),
                    crate::runtime::assets::Character {
                        id: id.clone(),
                        name: name.clone(),
                        image_tag: image_tag.clone(),
                        voice_tag: voice_tag.clone(),
                    },
                );
            }
            _ => {}
        }
    }
}


fn pre_narration_lines(body: &mut Vec<Stmt>) {
    let mut new_body = Vec::new();
    for stmt in body.drain(..) {
        match stmt { 
            Stmt::Narration {span, lines} => {
                for l in lines {
                    new_body.push(Stmt::Narration {span, lines: vec![l]});
                }
            },
            Stmt::Label { span, id, mut body } => {
                pre_narration_lines(&mut body);
                new_body.push(Stmt::Label { span, id, body });
            },
            Stmt::Choice { span, title, mut arms, id } => {
                for arm in &mut arms {
                    pre_narration_lines(&mut arm.body);
                }
                new_body.push(Stmt::Choice { span, title, arms, id });
            }
            Stmt::If { span, mut branches, mut else_branch, id } => {
                for (_, body) in &mut branches {
                    pre_narration_lines(body);
                }
                if let Some(body) = &mut else_branch {
                    pre_narration_lines(body);
                }
                new_body.push(Stmt::If { span, branches, else_branch, id });
            }
            _ => new_body.push(stmt),
        }
    }
    *body = new_body;
}

fn preprocess_block(
    stmts: &mut [Stmt],
    scope_name: &str,
    map: &mut FxHashMap<String, Arc<[Stmt]>>
) {
    let mut counters: HashMap<&str, usize> = HashMap::new();

    for stmt in stmts {
        match stmt {
            Stmt::Label {id, body, ..} => {
                preprocess_block(body, id, map);
                map.insert(id.clone(), Arc::from(body.as_slice()));
            },
            Stmt::If { branches, else_branch, id, .. } => {
                let count = counters.entry("if").or_insert(0);
                let base_id = format!("{}@if_{}", scope_name, count);
                *count += 1;
                *id = Some(base_id.clone());

                for (idx, (_, body)) in branches.iter_mut().enumerate() {
                    let branch_id = format!("{}_b{}", base_id, idx);
                    preprocess_block(body, &branch_id, map);
                    map.insert(branch_id.clone(), Arc::from(body.as_slice()));
                }

                if let Some(body) = else_branch {
                    let branch_id = format!("{}_else", base_id);
                    preprocess_block(body, &branch_id, map);
                    map.insert(branch_id.clone(), Arc::from(body.as_slice()));
                }
            },
            Stmt::Choice { arms, id, .. } => {
                let count = counters.entry("choice").or_insert(0);
                let base_id = format!("{}@choice_{}", scope_name, count);
                *count += 1;
                *id = Some(base_id.clone());

                for (idx, arm) in arms.iter_mut().enumerate() {
                    let arm_id = format!("{}_opt{}", base_id, idx);
                    preprocess_block(&mut arm.body, &arm_id, map);
                    map.insert(arm_id.clone(), Arc::from(arm.body.as_slice()));
                }
            },
            _ => {}
        }
    }
}