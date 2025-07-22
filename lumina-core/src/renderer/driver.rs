use crate::renderer::Renderer;
use crate::executor::Executor;
use crate::event::EngineEvent;
use viviscript_core::ast::Script;
use crate::Ctx;

pub struct Driver<R: Renderer> {
    exe: Executor,
    renderer: R,
}

impl<R: Renderer> Driver<R> {
    pub fn new(ctx: &mut Ctx, ast: &Script, renderer: R) -> Self {
        let mut exe = Executor::new();
        exe.start(ctx, &ast, "init");
        Driver { exe, renderer }
    }
    
    pub fn run(&mut self, ctx: &mut Ctx, ast: &Script) {
        loop {
            let waiting = self.exe.step(ctx, &ast);
            println!("{:?}", ctx.layer_record);
            for ev in ctx.drain() {
                if let Some(reply) = self.renderer.handle(&ev) {
                    self.exe.feed(reply);
                }
                if matches!(ev, EngineEvent::End) {
                    return;
                }
            }

            if !waiting {
                continue;
            }
        }
    }
}