use std::sync::Arc;
use viviscript_core::ast::Script;
use crate::{storager, Ctx, Executor};
use crate::event::InputEvent;

pub struct ExecutorHandle{
    exe: Executor,
    script: Arc<Script>,
}

impl ExecutorHandle {
    pub fn new(ctx: &mut Ctx, script: Arc<Script>) -> Self {
        let mut exe = Executor::new();
        exe.load_script(ctx, script.clone());
        exe.start(ctx, "init");
        Self { exe, script }
    }

    #[inline]
    pub fn step(&mut self, ctx: &mut Ctx) -> bool {
        self.exe.step(ctx)
    }

    #[inline]
    pub fn feed(&mut self, ctx: &mut Ctx, ev: InputEvent) {
        match ev {
            InputEvent::SaveRequest {slot} => {
                log::info!("Try to save request slot: {}", slot);
                storager::save(&format!("save{}.bin", slot), ctx.clone(), self.exe.clone())
                    .unwrap_or_else(|e| log::error!("save failed: {}", e));
                self.exe.feed(InputEvent::Continue);
                log::info!("Save finished");
            }
            InputEvent::LoadRequest { slot } => {
                log::info!("Load request slot: {}", slot);
                if let Ok((new_ctx, new_exe)) = storager::load(&format!("save{}.bin", slot), self.script.clone()) {
                    *ctx = new_ctx;
                    ctx.dialogue_history.pop();
                    self.exe = new_exe;
                    log::info!("Load finished");
                }else {
                    log::warn!("load failed");
                }
            }
            _ => self.exe.feed(ev),
        }
    }
}