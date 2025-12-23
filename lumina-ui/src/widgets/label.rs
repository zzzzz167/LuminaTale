use crate::{Rect, Color, UiRenderer, Alignment};
pub struct Label<'a> {
    text: &'a str,
    color: Color,
    size: f32,
    align: Alignment,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            color: Color::WHITE,
            size: 24.0,
            align: Alignment::Start
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

    pub fn align(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) {
        ui.draw_text(self.text, rect, self.color, self.size, self.align);
    }
}