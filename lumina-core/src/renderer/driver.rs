use std::panic;
use log::log;
use crate::{
    event::{InputEvent, OutputEvent},
    renderer::Renderer,
    storager,
    Ctx, Executor,
};
use viviscript_core::ast::Script;


pub struct Driver<R: Renderer> {
    exe: Executor,
    renderer: R,
}

impl<R: Renderer> Driver<R> {
    pub fn new(ctx: &mut Ctx, ast: &mut Script, renderer: R) -> Self {
        let mut exe = Executor::new();
        exe.start(ctx, ast, "init");
        panic::set_hook(Box::new(|info| {
            let msg = info
                .payload()
                .downcast_ref()
                .unwrap_or(&"unknown panic payload");
            let location = info.location().map_or("unknown location".to_string(), |l|{
                format!("({}:{})", l.file(), l.line())
            });
            log::error!("{}{}", msg, location);
            std::process::exit(1);
        }));
        Driver { exe, renderer }
    }
    pub fn run(&mut self, ctx: &mut Ctx, ast: &Script) {
        loop {
            let waiting = self.exe.step(ctx, ast);
            for out in ctx.drain() {
                if let Some(inp) = self.renderer.render(&out, ctx) {
                    self.dispatch_input(ctx, ast, inp);
                }
                if matches!(out, OutputEvent::End) {
                    return;
                }
            }
            if waiting {
                if let Some(inp) = self.renderer.render(&OutputEvent::StepDone, ctx) {
                    self.dispatch_input(ctx, ast, inp);
                }
            }
        }
    }
    
    fn dispatch_input(&mut self, ctx: &mut Ctx, ast: &Script, inp: InputEvent) {
        match inp { 
            InputEvent::SaveRequest {slot} => {
                log::info!("Try to save request slot: {}", slot);
                storager::save(&format!("save{}.bin", slot), ctx.clone(), self.exe.clone())
                    .unwrap_or_else(|e| log::error!("save failed: {}", e));
                self.dispatch_input(ctx, ast, InputEvent::Continue);
                log::info!("Save finished");
            }
            InputEvent::LoadRequest { slot } => {
                log::info!("Load request slot: {}", slot);
                if let Ok((new_ctx, new_exe)) = storager::load(&format!("save{}.bin", slot), ast) {
                    *ctx = new_ctx;
                    ctx.dialogue_history.pop();
                    self.exe = new_exe;
                    log::info!("Load finished");
                }else { 
                    log::warn!("load failed");
                }
            }
            _ => self.exe.feed(inp),
        }
    }
}