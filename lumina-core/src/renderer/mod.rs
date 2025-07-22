pub mod terminal;
pub mod driver;

use crate::event::EngineEvent;
use crate::runtime::Ctx;

pub trait Renderer {
    fn handle(&mut self, ev: &EngineEvent) -> Option<EngineEvent>;
}