#[cfg(feature = "tui")]
pub mod terminal;
pub mod driver;

use viviscript_core::ast::Script;
use crate::Ctx;

pub trait Renderer {
    fn run_event_loop(&mut self, ctx: &mut Ctx, script: Script);
}