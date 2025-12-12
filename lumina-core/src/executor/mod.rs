mod frame;
mod call_stack;
mod walk;

use std::rc::Rc;
use rustc_hash::FxHashMap;
use mlua::Lua;
use viviscript_core::ast::{Stmt, Script};
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
    pending_choice: Option<Vec<Vec<Stmt>>>,
    pause: bool,
    label_map: FxHashMap<String, Rc<[Stmt]>>,
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

    pub fn preload_script(&mut self, ctx: &mut Ctx ,script: &mut Script) {
        log::info!("Processing preload");
        pre_narration_lines(&mut script.body);
        pre_collect_characters(ctx, &script.body);
        // 建立 label → body 映射
        self.label_map.clear();
        build_label_map(&script.body, &mut self.label_map);
    }
    
    pub fn start(&mut self, ctx: &mut Ctx, label: &str) {
        init_ctx_runtime(ctx);
        let body = self.label_body(label).unwrap_or_else(|| panic!("label {} not found",label));
        self.call_stack.push(Frame::new(label.to_string(),body.to_vec(), 0));
    }
    pub fn feed(&mut self, ev: InputEvent) {
        match ev {
            InputEvent::ChoiceMade { index } => {
                if let Some(mut arms) = self.pending_choice.take() {
                    if index < arms.len() {
                        let selected_body = arms.remove(index);

                        let frame = self.call_stack.top_mut().unwrap();
                        frame.advance();

                        let return_frame = Frame::new(frame.name.clone(), frame.stmts.clone(), frame.pc);
                        self.call_stack.pop();
                        self.call_stack.push(return_frame);

                        self.call_stack.push(Frame::new("__choice_block__".to_string(), selected_body, 0));
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
            if let Some(body) = self.label_body(&fs.label) {
                if fs.pc > body.len() {
                    panic!("saved pc {} out of range for label {}", fs.pc, fs.label);
                }
                let frame = Frame::new(fs.label, body, fs.pc);
                self.call_stack.push(frame);
            } else {
                log::warn!("Cannot restore frame {}, label not found (dynamic block?)", fs.label);
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

    fn label_body(&self, name: &str) -> Option<&[Stmt]> {
        self.label_map.get(name).map(|rc| rc.as_ref())
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
        let body = self.label_body(label)
            .unwrap_or_else(|| panic!("label {} not found", label))
            .to_vec();
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
                let body = self
                    .label_body(&target)
                    .unwrap_or_else(|| panic!("label {} not found",target))
                    .to_vec();
                let frame = self.call_stack.top_mut().unwrap();
                let return_frame = Frame::new(frame.name.clone(),frame.stmts.clone(), frame.pc + 1);
                self.call_stack.pop();
                self.call_stack.push(return_frame);
                self.call_stack.push(Frame::new(target,body, 0));
            },
            NextAction::EnterBlock(stmts) => {
                let frame = self.call_stack.top_mut().unwrap();
                let return_frame = Frame::new(frame.name.clone(), frame.stmts.clone(), frame.pc + 1);

                self.call_stack.pop();
                self.call_stack.push(return_frame);

                self.call_stack.push(Frame::new("__if_block__".to_string(), stmts, 0));
            }
        }
    }
}

fn build_label_map(stmts: &[Stmt], map: &mut FxHashMap<String, Rc<[Stmt]>>) {
    for stmt in stmts {
        match stmt {
            Stmt::Label { id, body, .. } => {
                map.insert(id.clone(), Rc::from(body.as_slice()));
                build_label_map(body, map);
            }
            Stmt::Choice { arms, .. } => {
                for arm in arms {
                    build_label_map(&arm.body, map);
                }
            }
            Stmt::If { branches, else_branch, .. } => {
                for (_, body) in branches {
                    build_label_map(body, map);
                }
                if let Some(body) = else_branch {
                    build_label_map(body, map);
                }
            }
            _ => {}
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
            Stmt::Choice { span, title, mut arms } => {
                for arm in &mut arms {
                    pre_narration_lines(&mut arm.body);
                }
                new_body.push(Stmt::Choice { span, title, arms });
            }
            Stmt::If { span, mut branches, mut else_branch } => {
                for (_, body) in &mut branches {
                    pre_narration_lines(body);
                }
                if let Some(body) = &mut else_branch {
                    pre_narration_lines(body);
                }
                new_body.push(Stmt::If { span, branches, else_branch });
            }
            _ => new_body.push(stmt),
        }
    }
    *body = new_body;
}