use crate::{Rect, Color, UiRenderer};
use crate::input::Interaction;

pub struct Slider<'a> {
    value: &'a mut f32, // 直接修改外部数据
    min: f32,
    max: f32,
    bar_color: Color,
    fill_color: Color,
    knob_color: Color,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            bar_color: Color::rgb(60, 60, 60),  // 深灰底槽
            fill_color: Color::rgb(100, 180, 255), // 亮蓝填充
            knob_color: Color::WHITE,           // 白色圆钮
        }
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) -> bool {
        // 1. 交互检测
        let interaction = ui.interact(rect);
        let mut changed = false;

        // 2. 核心逻辑：如果被按住，根据鼠标位置计算新值
        if interaction == Interaction::Held || interaction == Interaction::Clicked {
            let (mx, _my) = ui.cursor_pos();

            // 计算比例 (0.0 ~ 1.0)
            let ratio = (mx - rect.x) / rect.w;
            let ratio = ratio.clamp(0.0, 1.0);

            // 映射到 min ~ max
            let new_value = self.min + ratio * (self.max - self.min);

            if *self.value != new_value {
                *self.value = new_value;
                changed = true;
            }
        }

        // 3. 绘制逻辑
        // A. 绘制底槽 (居中细条)
        let bar_height = 4.0;
        let bar_y = rect.y + (rect.h - bar_height) / 2.0;
        let bar_rect = Rect::new(rect.x, bar_y, rect.w, bar_height);
        ui.draw_rect(bar_rect, self.bar_color);

        // B. 计算当前值的比例
        let current_ratio = (*self.value - self.min) / (self.max - self.min);
        let current_ratio = current_ratio.clamp(0.0, 1.0);

        // C. 绘制已填充部分
        let fill_width = rect.w * current_ratio;
        let fill_rect = Rect::new(rect.x, bar_y, fill_width, bar_height);
        ui.draw_rect(fill_rect, self.fill_color);

        // D. 绘制圆钮 (Knob)
        let knob_x = rect.x + fill_width;
        let knob_y = rect.y + rect.h / 2.0;
        let radius = if interaction == Interaction::Held { 10.0 } else { 8.0 }; // 按住时变大
        ui.draw_circle((knob_x, knob_y), radius, self.knob_color);

        changed
    }
}