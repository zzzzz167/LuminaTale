use crate::{Rect, UiRenderer, Style, Background, Color, Border, GradientDirection};

pub struct Panel {
    style: Style,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            style: Style {
                // 默认还是半透明黑
                background: Background::Solid(Color::rgba(0, 0, 0, 200)),
                border: Border::default(),
            }
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.style.background = Background::Solid(color);
        self
    }

    pub fn gradient(mut self, dir: GradientDirection, start: Color, end: Color) -> Self {
        self.style.background = Background::LinearGradient {
            dir,
            colors: (start, end),
        };
        self
    }

    pub fn image(mut self, id: &str) -> Self {
        self.style.background = Background::Image(id.to_string());
        self
    }

    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.style.border.color = color;
        self.style.border.width = width;
        self
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.style.border.radius = radius;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) {
        ui.draw_style(rect, &self.style);
    }
}