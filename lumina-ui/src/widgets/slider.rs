use crate::{Rect, Color, UiRenderer, Style, Background};
use crate::input::Interaction;

pub struct Slider<'a> {
    value: &'a mut f32, // 直接修改外部数据
    min: f32,
    max: f32,
    track_style: Style,
    fill_style: Style,
    knob_style: Style,

    knob_size: f32,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, min: f32, max: f32) -> Self {
        let mut track = Style::default();
        track.background = Background::Solid(Color::rgb(60, 60, 60));
        track.border.radius = 2.0;

        let mut fill = Style::default();
        fill.background = Background::Solid(Color::rgb(100, 180, 255));
        fill.border.radius = 2.0;

        let mut knob = Style::default();
        knob.background = Background::Solid(Color::WHITE);
        knob.border.radius = 10.0;

        Self {
            value,
            min,
            max,
            track_style: track,
            fill_style: fill,
            knob_style: knob,
            knob_size: 20.0,
        }
    }

    pub fn style_track(mut self, style: Style) -> Self {
        self.track_style = style;
        self
    }

    pub fn style_fill(mut self, style: Style) -> Self {
        self.fill_style = style;
        self
    }

    pub fn style_knob(mut self, style: Style, size: f32) -> Self {
        self.knob_style = style;
        self.knob_size = size;
        self
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) -> bool {
        let interaction = ui.interact(rect);
        let mut changed = false;

        if interaction == Interaction::Held || interaction == Interaction::Clicked {
            let (mx, _my) = ui.cursor_pos();
            let ratio = (mx - rect.x) / rect.w;
            let ratio = ratio.clamp(0.0, 1.0);
            let new_value = self.min + ratio * (self.max - self.min);
            if *self.value != new_value {
                *self.value = new_value;
                changed = true;
            }
        }

        // 1. 绘制轨道 (垂直居中)
        let bar_height = 6.0; // 稍微粗一点
        let bar_y = rect.y + (rect.h - bar_height) / 2.0;
        let track_rect = Rect::new(rect.x, bar_y, rect.w, bar_height);
        ui.draw_style(track_rect, &self.track_style);

        // 2. 绘制填充条
        let current_ratio = (*self.value - self.min) / (self.max - self.min);
        let current_ratio = current_ratio.clamp(0.0, 1.0);
        let fill_width = rect.w * current_ratio;
        let fill_rect = Rect::new(rect.x, bar_y, fill_width, bar_height);
        ui.draw_style(fill_rect, &self.fill_style);

        // 3. 绘制滑块 (Knob)
        // 计算滑块中心点
        let knob_center_x = rect.x + fill_width;
        let knob_center_y = rect.y + rect.h / 2.0;

        // 计算滑块矩形 (以中心点为基准)
        let half_size = self.knob_size / 2.0;
        let knob_rect = Rect::new(
            knob_center_x - half_size,
            knob_center_y - half_size,
            self.knob_size,
            self.knob_size
        );

        ui.draw_style(knob_rect, &self.knob_style);

        changed
    }
}