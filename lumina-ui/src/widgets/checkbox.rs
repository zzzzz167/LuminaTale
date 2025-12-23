use crate::{Rect, Color, UiRenderer};
use crate::input::Interaction;

pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: &'a str,
    size: f32,
    color: Color,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, label: &'a str) -> Self {
        Self {
            checked,
            label,
            size: 24.0,
            color: Color::WHITE,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) -> bool {
        let interaction = ui.interact(rect);

        // 1. 交互逻辑：点击反转状态
        let mut changed = false;
        if interaction.is_clicked() {
            *self.checked = !*self.checked;
            changed = true;
        }

        // 2. 布局计算
        // 方框区域 (正方形)
        let box_size = self.size;
        // 垂直居中
        let box_y = rect.y + (rect.h - box_size) / 2.0;
        let box_rect = Rect::new(rect.x, box_y, box_size, box_size);

        // 3. 绘制方框
        ui.draw_border(box_rect, self.color, 2.0);

        // 4. 如果选中，画里面的钩钩
        if *self.checked {
            // 内缩一点点
            let inner_rect = box_rect.shrink(4.0);
            ui.draw_rect(inner_rect, self.color);
        }

        // 5. 绘制文字 (在方框右边)
        let text_x = rect.x + box_size + 10.0;
        let text_w = rect.w - (box_size + 10.0);
        let text_rect = Rect::new(text_x, rect.y, text_w, rect.h);
        
        ui.draw_text(self.label, text_rect, self.color, self.size);

        changed
    }
}