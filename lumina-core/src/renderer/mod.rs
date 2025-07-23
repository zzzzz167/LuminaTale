pub mod terminal;
pub mod driver;

use crate::Ctx;
use crate::event::{InputEvent, OutputEvent};

pub trait Renderer {
    fn render(&mut self, out: &OutputEvent, ctx: &mut Ctx) -> Option<InputEvent>;
}