use crate::ui::UiDrawer;
use crate::core::{AssetManager, Painter, AudioPlayer};
use crate::screens::{Screen, ScreenTransition};
use lumina_core::Ctx;
use lumina_ui::{Rect, Color, Alignment, Style, Background, Border, GradientDirection};
use lumina_ui::widgets::{Button, Label, Panel, Slider, Checkbox};
use winit::event_loop::ActiveEventLoop;

pub struct SettingsScreen {
    // 模拟的设置状态
    bgm_volume: f32,
    se_volume: f32,
    fullscreen: bool,
    auto_mode: bool,

    // 退出标识
    should_close: bool,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            bgm_volume: 0.5,
            se_volume: 0.8,
            fullscreen: false,
            auto_mode: true,
            should_close: false,
        }
    }
}

impl Screen for SettingsScreen {
    fn update(
        &mut self,
        _dt: f32,
        _ctx: &mut Ctx,
        _el: &ActiveEventLoop,
        _assets: &AssetManager,
        _audio: &mut AudioPlayer
    ) -> ScreenTransition {
        if self.should_close {
            return ScreenTransition::Pop; // 返回上一层 (主菜单)
        }
        ScreenTransition::None
    }

    fn draw(&mut self, ui: &mut UiDrawer, _painter: &mut Painter, rect: Rect, _ctx: &mut Ctx) {
        // 1. 半透明黑色背景遮罩 (覆盖在主菜单之上)
        Panel::new()
            .color(Color::rgba(0, 0, 0, 220))
            .show(ui, rect);

        // 2. 居中设置面板
        let panel_rect = rect.center(600.0, 500.0);

        // 面板背景：深灰 -> 黑色垂直渐变，带边框和圆角
        Panel::new()
            .gradient(
                GradientDirection::Vertical,
                Color::rgb(60, 60, 70),
                Color::rgb(30, 30, 40)
            )
            .stroke(Color::rgb(100, 100, 120), 2.0)
            .rounded(16.0)
            .show(ui, panel_rect);

        // 3. 布局内容
        let content = panel_rect.shrink(40.0);
        let (header, body) = content.split_top(60.0);

        // 标题
        Label::new("SETTINGS")
            .size(40.0)
            .align(Alignment::Center)
            .show(ui, header);

        // 分割各项 (每一行高 80px)
        let (row_bgm, rest) = body.split_top(80.0);
        let (row_se, rest) = rest.split_top(80.0);
        let (row_check1, rest) = rest.split_top(60.0);
        let (row_check2, rest) = rest.split_top(60.0);
        let (row_btn, _) = rest.split_bottom(60.0); // 底部放按钮

        // --- 示例 1: 标准 Slider (BGM) ---
        let (label_rect, slider_rect) = row_bgm.shrink(10.0).split_left(150.0);
        Label::new("BGM Volume").align(Alignment::Start).show(ui, label_rect);

        Slider::new(&mut self.bgm_volume, 0.0, 1.0)
            .show(ui, slider_rect); // 使用默认样式

        // --- 示例 2: 高度自定义 Slider (SE) ---
        // 演示：红黑渐变轨道 + 方形滑块
        let (label_rect, slider_rect) = row_se.shrink(10.0).split_left(150.0);
        Label::new("SE Volume").align(Alignment::Start).show(ui, label_rect);

        // 自定义轨道样式
        let mut custom_track = Style::default();
        custom_track.background = Background::LinearGradient {
            dir: GradientDirection::Horizontal,
            colors: (Color::BLACK, Color::rgb(150, 0, 0))
        };
        custom_track.border.radius = 4.0;

        // 自定义滑块样式 (红色正方形，小白边)
        let mut custom_knob = Style::default();
        custom_knob.background = Background::Solid(Color::RED);
        custom_knob.border = Border { color: Color::WHITE, width: 2.0, radius: 2.0 };

        Slider::new(&mut self.se_volume, 0.0, 1.0)
            .style_track(custom_track)
            .style_knob(custom_knob, 24.0) // 24px 大小的滑块
            .show(ui, slider_rect);

        // --- 示例 3: 标准 Checkbox ---
        Checkbox::new(&mut self.fullscreen, "Fullscreen Mode")
            .show(ui, row_check1.shrink(10.0));

        // --- 示例 4: 自定义样式 Checkbox ---
        // 未选中是红框，选中是绿框+实心
        let mut style_unchecked = Style::default();
        style_unchecked.border = Border { color: Color::RED, width: 2.0, radius: 8.0 };

        let mut style_checked = Style::default();
        style_checked.background = Background::Solid(Color::GREEN);
        style_checked.border = Border { color: Color::WHITE, width: 2.0, radius: 8.0 };

        Checkbox::new(&mut self.auto_mode, "Auto Play (Custom)")
            .style_unchecked(style_unchecked)
            .style_checked(style_checked)
            // .font("pixel") // 如果你有自定义字体
            .show(ui, row_check2.shrink(10.0));

        // --- 关闭按钮 ---
        if Button::new("Close")
            // 自定义常态
            .style_normal(Style {
                background: Background::Solid(Color::rgb(80, 80, 100)),
                border: Border { radius: 8.0, ..Default::default() }
            })
            // 自定义悬停态 (变亮 + 白边)
            .style_hover(Style {
                background: Background::Solid(Color::rgb(100, 100, 120)),
                border: Border { radius: 8.0, color: Color::WHITE, width: 2.0 }
            })
            .show(ui, row_btn.center(120.0, 50.0))
        {
            self.should_close = true;
        }
    }
}