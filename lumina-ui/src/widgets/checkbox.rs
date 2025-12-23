use crate::{Alignment, Background, Border, Color, Rect, Style, UiRenderer};

pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: &'a str,
    size: f32,
    unchecked_style: Style,
    checked_style: Style,
    text_color: Color,
    font: Option<&'a str>,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, label: &'a str) -> Self {
        let mut unchecked = Style::default();
        unchecked.border = Border { color: Color::WHITE, width: 2.0, radius: 4.0 };

        let mut checked_style = Style::default();
        checked_style.background = Background::Solid(Color::WHITE);
        checked_style.border.radius = 4.0;

        Self {
            checked,
            label,
            size: 24.0,
            unchecked_style: unchecked,
            checked_style,
            text_color: Color::WHITE,
            font: None,
        }
    }

    // --- 样式配置 ---

    /// 设置“未选中”时的样式 (例如：空盒子图片)
    pub fn style_unchecked(mut self, style: Style) -> Self {
        self.unchecked_style = style;
        self
    }

    /// 设置“选中”时的样式 (例如：打钩图片)
    pub fn style_checked(mut self, style: Style) -> Self {
        self.checked_style = style;
        self
    }

    /// 快捷设置：图片 Checkbox
    pub fn images(mut self, unchecked_id: String, checked_id: String) -> Self {
        self.unchecked_style.background = Background::Image(unchecked_id);
        self.unchecked_style.border.width = 0.0; // 用图了就去掉边框

        self.checked_style.background = Background::Image(checked_id);
        self.checked_style.border.width = 0.0;
        self
    }

    pub fn font(mut self, font: &'a str) -> Self {
        self.font = Some(font);
        self
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) -> bool {
        let interaction = ui.interact(rect);
        let mut changed = false;
        if interaction.is_clicked() {
            *self.checked = !*self.checked;
            changed = true;
        }

        let box_size = self.size;
        let box_y = rect.y + (rect.h - box_size) / 2.0;
        let box_rect = Rect::new(rect.x, box_y, box_size, box_size);

        // 根据状态选择样式
        let current_style = if *self.checked {
            &self.checked_style
        } else {
            &self.unchecked_style
        };

        ui.draw_style(box_rect, current_style);

        // 文字
        let text_x = rect.x + box_size + 10.0;
        let text_w = rect.w - (box_size + 10.0);
        let text_rect = Rect::new(text_x, rect.y, text_w, rect.h);

        ui.draw_text(self.label, text_rect, self.text_color, self.size, Alignment::Center, self.font);

        changed
    }
}