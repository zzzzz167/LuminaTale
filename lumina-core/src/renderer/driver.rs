use std::sync::Arc;
use crate::{storager, Ctx, Executor};
use crate::event::InputEvent;
use crate::manager::ScriptManager;

pub struct ExecutorHandle{
    exe: Executor,
    manager: Arc<ScriptManager>,
}

impl ExecutorHandle {
    pub fn new(ctx: &mut Ctx, manager: Arc<ScriptManager>) -> Self {
        let mut exe = Executor::new(manager.clone());
        exe.start(ctx, "init");
        Self { exe, manager }
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
                if let Ok((new_ctx, new_exe)) = storager::load(&format!("save{}.bin", slot), self.manager.clone()) {
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