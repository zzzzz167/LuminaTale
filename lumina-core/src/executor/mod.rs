mod frame;
mod call_stack;
mod walk;

use crate::runtime::Ctx;
use mlua::Lua;
use viviscript_core::ast::{Stmt, Script};
use frame::Frame;
use call_stack::CallStack;
use crate::event::{EngineEvent, Mode};
use crate::executor::walk::{walk_stmt, NextAction, StmtEffect};


#[derive(Debug)]
pub struct Executor {
    call_stack: CallStack,
    lua: Lua,
    // 当遇到 Choice 时暂存分支，等待 ChoiceMade
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
    pub fn start(&mut self, ctx: &mut Ctx ,script: &Script, label: &str) {
        preload(ctx, script);
        let body = find_label_body(script, label).unwrap_or_else(|| panic!("label {} not found",label));
        self.call_stack.push(Frame::new(body.to_vec(), 0));
    }
    pub fn feed(&mut self, ev: EngineEvent) {
        match ev {
            EngineEvent::ChoiceMade { index } => {
                if let Some(arms) = self.pending_choice.take() {
                    if let Some(frame) = self.call_stack.top_mut(){
                        frame.advance();
                    }
                    self.call_stack.push(Frame::new(arms[index].clone(), 0));
                }
            },
            EngineEvent::InputMode { mode } => {
                match mode {
                    Mode::Exit => {
                        self.call_stack.clear();
                    },
                    Mode::Continue => {}
                }
                self.pause = false;
                if let Some(frame) = self.call_stack.top_mut(){
                    frame.advance();
                }
            },
            _ => {}
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
            ctx.push(EngineEvent::End);
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
                self.call_stack.push(Frame::new(body, 0));
            },
            NextAction::Call(target) => {
                let body = find_label_body(script, &target).unwrap_or_else(|| panic!("label {} not found",target));
                let frame = self.call_stack.top_mut().unwrap();
                let return_frame = Frame::new(frame.stmts.clone(), frame.pc + 1);
                self.call_stack.pop();
                self.call_stack.push(return_frame);
                self.call_stack.push(Frame::new(body, 0));
            }
        }
    }
}

fn preload(ctx: &mut Ctx, node: &Script) {
    pre_collect_characters(ctx, &node.body);
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

fn find_label_body<'a>(script: &'a Script, name: &str) -> Option<&'a [Stmt]> {
    script.body.iter().find_map(|s| match s {
        Stmt::Label { id, body, .. } if id == name => Some(&body[..]),
        _ => None,
    })
}