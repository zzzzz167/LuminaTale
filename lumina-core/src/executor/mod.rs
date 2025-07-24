mod frame;
mod call_stack;
mod walk;

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
    pause: bool
}

impl Executor {
    pub fn new() -> Self{
        Executor {
            call_stack: CallStack::default(),
            lua:Lua::new(),
            pending_choice: None,
            pause: false
        }
    }
    pub fn start(&mut self, ctx: &mut Ctx ,script: &mut Script, label: &str) {
        preload(ctx, script);
        let body = find_label_body(script, label).unwrap_or_else(|| panic!("label {} not found",label));
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

    pub fn restore(&mut self, script: &Script, snap: Vec<FrameSnapshot>) {
        self.call_stack.clear();
        for fs in snap {
            let body = find_label_body(script, &fs.label)
                .unwrap_or_else(|| panic!("label {} not found", fs.label));
            if fs.pc > body.len() {
                panic!("saved pc {} out of range for label {}", fs.pc, fs.label);
            }
            let frame = Frame::new(fs.label,body, fs.pc);
            self.call_stack.push(frame);
        }
    }

    pub fn step(&mut self, ctx: &mut Ctx, script: &Script) -> bool {
        if self.pending_choice.is_some() || self.pause {
            return true;
        }
        if let Some(frame) = self.call_stack.top_mut() {
            if let Some(_) = frame.current() {
                self.exec_current(ctx, script);
            } else {
                self.call_stack.pop();
            }
            false
        } else {
            ctx.push(OutputEvent::End);
            false
        }
    }
    fn exec_current(&mut self, ctx: &mut Ctx,script: &Script) {
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
                let body = find_label_body(script, &label).unwrap_or_else(|| panic!("label {} not found",label));
                self.call_stack.clear();
                self.call_stack.push(Frame::new(label,body, 0));
            },
            NextAction::Call(target) => {
                let body = find_label_body(script, &target).unwrap_or_else(|| panic!("label {} not found",target));
                let frame = self.call_stack.top_mut().unwrap();
                let return_frame = Frame::new(frame.name.clone(),frame.stmts.clone(), frame.pc + 1);
                self.call_stack.pop();
                self.call_stack.push(return_frame);
                self.call_stack.push(Frame::new(target,body, 0));
            }
        }
    }
}

fn preload(ctx: &mut Ctx, node: &mut Script) {
    log::info!("Processing preload");
    pre_collect_characters(ctx, &node.body);
    pre_choice_labels(node);
    pre_narration_lines(&mut node.body);
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

fn find_label_body<'a>(script: &'a Script, name: &str) -> Option<&'a [Stmt]> {
    for stmt in &script.body {
        if let Some(body) = find_in_stmt(stmt, name) {
            return Some(body);
        }
    }
    None
}

fn find_in_stmt<'a>(stmt: &'a Stmt, name: &str) -> Option<&'a [Stmt]> {
    match stmt {
        Stmt::Label { id, body, .. } if id == name => Some(body),
        Stmt::Label { body, .. } => {
            // 继续往这个 label 的 body 里找
            body.iter().find_map(|s| find_in_stmt(s, name))
        }
        Stmt::Choice { arms, .. } => {
            arms.iter()
                .find_map(|arm| arm.body.iter().find_map(|s| find_in_stmt(s, name)))
        }
        _ => None,
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