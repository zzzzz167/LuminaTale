use crate::{Rect, Color, UiRenderer};

pub struct Image<'a> {
    id: &'a str,
    tint: Color,
}

impl<'a> Image<'a> {
    pub fn new(id: &'a str) -> Self {
        Self {
            id,
            tint: Color::WHITE,
        }
    }

    pub fn tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) {
        ui.draw_image(self.id, rect, self.tint);
    }
}