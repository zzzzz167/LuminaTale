use crate::{Rect, Color, UiRenderer};
pub struct Label<'a> {
    text: &'a str,
    color: Color,
    size: f32,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            color: Color::WHITE,
            size: 24.0,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) {
        ui.draw_text(self.text, rect, self.color, self.size);
    }
}