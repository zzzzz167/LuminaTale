mod frame;
mod call_stack;
mod walk;

use std::rc::Rc;
use rustc_hash::FxHashMap;
use crate::runtime::Ctx;
use viviscript_core::lexer::Span;
use mlua::Lua;
use viviscript_core::ast::{Stmt, Script};
use frame::Frame;
use call_stack::CallStack;
use crate::event::{OutputEvent, InputEvent};
use crate::executor::walk::{walk_stmt, NextAction, StmtEffect};
use crate::storager::types::FrameSnapshot;

#[derive(Debug, Clone)]
pub struct Executor {
    call_stack: CallStack,
    lua: Lua,
    pending_choice: Option<Vec<Vec<Stmt>>>,
    pause: bool,
    label_map: FxHashMap<String, Rc<[Stmt]>>,
}

impl Executor {
    pub fn new() -> Self{
        Executor {
            call_stack: CallStack::default(),
            lua:Lua::new(),
            pending_choice: None,
            pause: false,
            label_map: FxHashMap::default(),
        }
    }

    pub fn preload_script(&mut self, ctx: &mut Ctx ,script: &mut Script) {
        log::info!("Processing preload");
        pre_choice_labels(script);
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
                if let Some(arms) = self.pending_choice.take() {
                    let frame = self.call_stack.top_mut().unwrap();
                    frame.advance();
                    let label = arms[index].clone();
                    if label.len() == 1{
                        match &label[0] {
                            Stmt::Label {id, body,..}=>{
                                self.call_stack.push(Frame::new(id.to_string(),body.to_vec(),0));
                            },
                            _ => {}
                        }
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
            let body = self.label_body(&fs.label)
                .unwrap_or_else(|| panic!("label {} not found", fs.label));
            if fs.pc > body.len() {
                panic!("saved pc {} out of range for label {}", fs.pc, fs.label);
            }
            let frame = Frame::new(fs.label,body, fs.pc);
            self.call_stack.push(frame);
        }
    }

    pub fn step(&mut self, ctx: &mut Ctx) -> bool {
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
                let body = self
                    .label_body(&label)
                    .unwrap_or_else(|| panic!("label {} not found", label))
                    .to_vec();
                self.call_stack.clear();
                self.call_stack.push(Frame::new(label,body, 0));
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

fn pre_choice_labels(script: &mut Script) {
    let mut seq = 0usize;
    for stmt in &mut script.body {
        if let Stmt::Label {body, ..} = stmt {
            preprocess_in_body(body,&mut seq);
        }
    }

    fn preprocess_in_body(body: &mut Vec<Stmt>, seq: &mut usize) {
        for stmt in body {
            match stmt {
                Stmt::Choice {arms, ..}=> {
                    for (idx, arm) in arms.iter_mut().enumerate() {
                        let label = format!("__choice_{}_{}", *seq, idx);
                        let old_body = std::mem::take(&mut arm.body);
                        arm.body = vec![Stmt::Label {
                            span: Span {start:0, end:0, line:0},
                            id: label,
                            body: old_body,
                        }]
                    }
                    *seq += 1;
                },
                _ => {}
            }
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
            _ => new_body.push(stmt),
        }
    }
    *body = new_body;
}