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
        Driver { exe, renderer }
    }
    pub fn run(&mut self, ctx: &mut Ctx, ast: &Script) {
        loop {
            let waiting = self.exe.step(ctx, ast);
            for out in ctx.drain() {
                // 2. 渲染并尝试拿输入
                if let Some(inp) = self.renderer.render(&out) {
                    self.dispatch_input(ctx, ast, inp);
                }
                if matches!(out, OutputEvent::End) {
                    return;
                }
            }
            
            if waiting {
                if let Some(inp) = self.renderer.render(&OutputEvent::StepDone) {
                    self.dispatch_input(ctx, ast, inp);
                }
            }
        }
    }
    
    fn dispatch_input(&mut self, ctx: &mut Ctx, ast: &Script, inp: InputEvent) {
        match inp { 
            InputEvent::SaveRequest {slot} => {
                storager::save(&format!("save{}.bin", slot), ctx.clone(), self.exe.clone())
                    .unwrap_or_else(|e| eprintln!("save failed: {}", e));
                self.dispatch_input(ctx, ast, InputEvent::Continue);
            }
            InputEvent::LoadRequest { slot } => {
                if let Ok((new_ctx, new_exe)) = storager::load(&format!("save{}.bin", slot), ast) {
                    *ctx = new_ctx;
                    self.exe = new_exe;
                }
            }
            _ => self.exe.feed(inp),
        }
    }
}