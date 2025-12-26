mod frame;
mod call_stack;
mod walk;
mod scanner;

use std::sync::Arc;
use mlua::Lua;
use viviscript_core::ast::Stmt;
use frame::Frame;
use call_stack::CallStack;

use crate::runtime::Ctx;
use crate::config::GraphicsConfig;
use crate::event::{OutputEvent, InputEvent};
use crate::executor::walk::{walk_stmt, NextAction, StmtEffect};
use crate::lua_glue::{self, CommandBuffer, LuaCommand};
use crate::storager::types::FrameSnapshot;
use crate::manager::ScriptManager;

#[derive(Clone)]
pub struct Executor {
    call_stack: CallStack,
    lua: Lua,
    cmd_buffer: CommandBuffer,
    pending_choice: Option<Vec<(String, Vec<Stmt>)>>,
    pause: bool,

    manager: Arc<ScriptManager>
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            .field("call_stack", &self.call_stack)
            .field("pause", &self.pause)
            .finish()
    }
}

impl Executor {
    pub fn new(manager: Arc<ScriptManager>) -> Self{
        let lua = Lua::new();
        let cmd_buffer = lua_glue::init_lua(&lua);

        Executor {
            call_stack: CallStack::default(),
            lua,
            cmd_buffer,
            pending_choice: None,
            pause: false,
            manager,
        }
    }
    
    pub fn start(&mut self, ctx: &mut Ctx, label: &str) {
        init_ctx_runtime(ctx);
        let global_chars = self.manager.collect_characters();
        ctx.characters.extend(global_chars);
        self.perform_jump(label);
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

    pub fn sync_vars_to_ctx(&self, ctx: &mut Ctx) {
        ctx.var_f = lua_glue::extract_vars(&self.lua);

        let sf_data = lua_glue::extract_sf(&self.lua);

        if let Err(e) = crate::storager::save_global("global.json", &sf_data) {
            log::error!("Failed to auto-save global.json: {}", e);
        } else {
            log::info!("Global data auto-saved.");
        }
    }

    pub fn sync_vars_from_ctx(&self, ctx: &mut Ctx) {
        lua_glue::inject_vars(&self.lua, &ctx.var_f);
    }

    pub fn load_global_data(&self) {
        match crate::storager::load_global("global.json") {
            Ok(data) => {
                if !data.is_null() {
                    log::info!("Global data loaded.");
                    lua_glue::inject_sf(&self.lua, &data);
                } else {
                    log::info!("No global data found (new game).");
                }
            }
            Err(e) => {
                // 只有真正的 IO 错误才报 Error，文件不存在是正常的
                log::warn!("Check global data: {}", e);
            }
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
                log::error!("Restore failed: Code block '{}' not found in project.", fs.label);
                panic!("Save file mismatch or Script changed");
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
        self.manager.get_label(name)
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
                LuaCommand::SaveGlobal => {
                    log::info!("Lua requested global save.");
                    let sf_data = lua_glue::extract_sf(&self.lua);

                    if let Err(e) = crate::storager::save_global("global.json", &sf_data) {
                        log::error!("Failed to save global.json: {}", e);
                    } else {
                        log::info!("Global data saved successfully.");
                    }
                }
            }
        }
        true
    }

    fn perform_jump(&mut self, label: &str) {
        let body = self.get_block_arc(label)
            .unwrap_or_else(|| panic!("Label '{}' not found in project!", label));

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
                self.trigger_preload(ctx);
                self.pending_choice = Some(arms);
            },
            NextAction::WaitInput => {
                self.trigger_preload(ctx);
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

    fn trigger_preload(&mut self, ctx: &mut Ctx) {
        let gf_cfg: GraphicsConfig = lumina_shared::config::get("graphics");

        if let Some(frame) = self.call_stack.top_mut() {
            let (images, audios) = scanner::Scanner::scan(
                &frame.stmts,
                frame.pc + 1,
                gf_cfg.preload_ahead,
                ctx
            );

            if !images.is_empty() || !audios.is_empty() {
                ctx.push(OutputEvent::Preload { images, audios });
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