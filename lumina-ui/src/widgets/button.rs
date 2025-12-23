use crate::{Rect, Color, UiRenderer, Alignment};
use crate::input::Interaction;

pub struct Button<'a> {
    text: &'a str,
    bg_color: Option<Color>,
    bg_hover: Option<Color>,
    bg_active: Option<Color>,
    text_color: Color,
    border: Option<(Color, f32)>,
    font_size: f32,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            bg_color: Some(Color::DARK_GRAY),
            bg_hover: Some(Color::GRAY),
            bg_active: Some(Color::rgb(20, 20, 20)),
            text_color: Color::WHITE,
            border: None,
            font_size: 24.0,
        }
    }

    /// 设置基础背景色 (传入 Color::TRANSPARENT 可透明)
    pub fn fill(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        // 如果用户只设了 fill，我们智能地自动设置 hover 颜色（简单的变亮/变暗逻辑可以以后加）
        // 这里简单处理：重置 hover/active 以免颜色不搭，或者让用户自己设
        self
    }

    /// 彻底透明 (无背景)
    pub fn transparent(mut self) -> Self {
        self.bg_color = None;
        self.bg_hover = Some(Color::rgba(255, 255, 255, 30)); // 悬停时微微发白
        self.bg_active = Some(Color::rgba(255, 255, 255, 60));
        self
    }

    /// 设置文字颜色
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// 设置文字大小
    pub fn size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// 添加边框
    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.border = Some((color, width));
        self
    }

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) -> bool {
        // 1. 交互检测
        let interaction = ui.interact(rect);

        // 2. 计算当前状态的颜色
        let current_bg = match interaction {
            Interaction::Held | Interaction::Clicked => self.bg_active,
            Interaction::Hovered => self.bg_hover,
            Interaction::None => self.bg_color,
        };

        // 3. 绘制背景
        if let Some(color) = current_bg {
            ui.draw_rect(rect, color);
        }

        // 4. 绘制边框
        if let Some((color, width)) = self.border {
            ui.draw_border(rect, color, width);
        }

        // 5. 绘制文字 (简单居中)
        ui.draw_text(self.text, rect, self.text_color, self.font_size, Alignment::Center);

        // 6. 返回点击状态
        interaction.is_clicked()
    }
}