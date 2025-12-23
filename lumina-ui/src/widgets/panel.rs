use crate::{Rect, Color, UiRenderer};

pub struct Panel {
    color: Color,
    border: Option<(Color, f32)>,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            color: Color::rgba(0, 0, 0, 200),
            border: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.border = Some((color, width));
        self
    }

    /// 仅绘制背景，不处理内部子元素布局
    /// 用户可以在调用 show 后，在这个 rect 上继续画别的东西
    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) {
        ui.draw_rect(rect, self.color);

        if let Some((c, w)) = self.border {
            ui.draw_border(rect, c, w);
        }
    }
}