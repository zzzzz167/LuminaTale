#[cfg(feature = "tui")]
pub mod terminal;
pub mod driver;

use std::sync::Arc;
use crate::manager::ScriptManager;
use crate::Ctx;

pub trait Renderer {
    fn run_event_loop(&mut self, ctx: &mut Ctx, manager: Arc<ScriptManager>);
}