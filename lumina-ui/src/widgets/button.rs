use crate::{Rect, Color, UiRenderer, Alignment, Style, Background, Border};
use crate::input::Interaction;

pub struct Button<'a> {
    text: &'a str,
    
    normal_style: Style,
    hover_style: Style,
    active_style: Style,

    text_color: Color,
    font_size: f32,
    font: Option<&'a str>,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str) -> Self {
        // --- 默认样式初始化 ---
        // 默认：深灰背景
        let mut normal = Style::default();
        normal.background = Background::Solid(Color::DARK_GRAY);

        // 悬停：浅灰背景
        let mut hover = normal.clone();
        hover.background = Background::Solid(Color::GRAY);

        // 按下：深黑背景
        let mut active = normal.clone();
        active.background = Background::Solid(Color::rgb(20, 20, 20));

        Self {
            text,
            normal_style: normal,
            hover_style: hover,
            active_style: active,
            text_color: Color::WHITE,
            font_size: 24.0,
            font: None,
        }
    }

    // ==========================================
    //  快捷配置 (同时应用到所有状态，或设置基础态)
    // ==========================================

    /// 设置基础背景色
    pub fn fill(mut self, color: Color) -> Self {
        self.normal_style.background = Background::Solid(color);
        self
    }

    /// 设置为透明按钮
    pub fn transparent(mut self) -> Self {
        self.normal_style.background = Background::None;
        self.hover_style.background = Background::Solid(Color::rgba(255, 255, 255, 30)); // 微微发亮
        self.active_style.background = Background::Solid(Color::rgba(255, 255, 255, 60));
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

    /// 设置字体 (例如 "pixel", "msyh")
    pub fn font(mut self, font_name: &'a str) -> Self {
        self.font = Some(font_name);
        self
    }

    /// 设置边框 (同时应用到所有状态，保持形状一致)
    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        let border = Border { color, width, radius: self.normal_style.border.radius };
        self.normal_style.border = border;
        self.hover_style.border = border;
        self.active_style.border = border;
        self
    }

    /// 设置圆角 (同时应用到所有状态)
    pub fn rounded(mut self, radius: f32) -> Self {
        self.normal_style.border.radius = radius;
        self.hover_style.border.radius = radius;
        self.active_style.border.radius = radius;
        self
    }

    // ==========================================
    //  高级自定义 (分别设置各状态样式)
    // ==========================================

    pub fn style_normal(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    pub fn style_hover(mut self, style: Style) -> Self {
        self.hover_style = style;
        self
    }

    pub fn style_active(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }

    // ==========================================
    //  渲染逻辑
    // ==========================================

    pub fn show(self, ui: &mut impl UiRenderer, rect: Rect) -> bool {
        // 1. 获取交互状态
        let interaction = ui.interact(rect);

        // 2. 根据状态选择样式
        let current_style = match interaction {
            Interaction::Held | Interaction::Clicked => &self.active_style,
            Interaction::Hovered => &self.hover_style,
            Interaction::None => &self.normal_style,
        };

        // 3. 绘制样式盒子 (背景 + 边框)
        ui.draw_style(rect, current_style);

        // 4. 绘制文字 (支持自定义字体)
        ui.draw_text(
            self.text,
            rect,
            self.text_color,
            self.font_size,
            Alignment::Center,
            self.font // 传入字体
        );

        // 5. 返回点击结果
        interaction.is_clicked()
    }
}