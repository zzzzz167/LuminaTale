pub mod terminal;
pub mod driver;

use crate::event::{InputEvent, OutputEvent};

pub trait Renderer {
    fn render(&mut self, out: &OutputEvent) -> Option<InputEvent>;
}