use std::panic;
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
    script: Script,
}

impl<R: Renderer> Driver<R> {
    pub fn new(ctx: &mut Ctx, mut script: Script, renderer: R) -> Self {
        let mut exe = Executor::new();
        exe.preload_script(ctx, &mut script);
        exe.start(ctx, "init");
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
        Driver { exe, renderer, script }
    }
    pub fn run(&mut self, ctx: &mut Ctx) {
        loop {
            let waiting = self.exe.step(ctx);
            for out in ctx.drain() {
                if let Some(inp) = self.renderer.render(&out, ctx) {
                    self.dispatch_input(ctx, inp);
                }
                if matches!(out, OutputEvent::End) {
                    return;
                }
            }
            if waiting {
                if let Some(inp) = self.renderer.render(&OutputEvent::StepDone, ctx) {
                    self.dispatch_input(ctx, inp);
                }
            }
        }
    }
    
    fn dispatch_input(&mut self, ctx: &mut Ctx, inp: InputEvent) {
        match inp { 
            InputEvent::SaveRequest {slot} => {
                log::info!("Try to save request slot: {}", slot);
                storager::save(&format!("save{}.bin", slot), ctx.clone(), self.exe.clone())
                    .unwrap_or_else(|e| log::error!("save failed: {}", e));
                self.dispatch_input(ctx, InputEvent::Continue);
                log::info!("Save finished");
            }
            InputEvent::LoadRequest { slot } => {
                log::info!("Load request slot: {}", slot);
                if let Ok((new_ctx, new_exe)) = storager::load(&format!("save{}.bin", slot), &mut self.script) {
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