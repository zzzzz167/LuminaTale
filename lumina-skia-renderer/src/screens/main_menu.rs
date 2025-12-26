use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use lumina_core::manager::ScriptManager;

use super::{Screen, ScreenTransition};
use crate::screens::ingame::InGameScreen;
use crate::screens::settings::SettingsScreen;

use crate::ui::UiDrawer;
use crate::core::{AssetManager, Painter, AudioPlayer};
use lumina_core::Ctx;
use lumina_core::renderer::driver::ExecutorHandle;

use lumina_ui::{Rect, Color, GradientDirection, Alignment, Transform, UiRenderer};
use lumina_ui::widgets::{Button, Label, Panel};

pub struct MainMenuScreen {
    manager: Arc<ScriptManager>,
    // 暂存这一帧 UI 点击产生的跳转指令
    pending_transition: ScreenTransition,
}

impl MainMenuScreen {
    pub fn new(manager: Arc<ScriptManager>) -> Self {
        Self {
            manager,
            pending_transition: ScreenTransition::None,
        }
    }
}

impl Screen for MainMenuScreen {
    fn update(
        &mut self,
        _dt: f32,
        _ctx: &mut Ctx,
        _el: &ActiveEventLoop,
        _assets: &mut AssetManager,
        _audio: &mut AudioPlayer
    ) -> ScreenTransition {
        // 将 draw 中产生的跳转指令提取出来返回给 Renderer
        // 同时重置为 None
        std::mem::replace(&mut self.pending_transition, ScreenTransition::None)
    }

    fn draw(
        &mut self,
        ui: &mut UiDrawer,
        _painter: &mut Painter,
        rect: Rect,
        ctx: &mut Ctx
    ) {
        // 1. 绘制背景
        Panel::new()
            .gradient(
                GradientDirection::Vertical,
                Color::rgb(20, 20, 30), // 深蓝黑
                Color::rgb(40, 30, 60)  // 紫黑
            )
            .show(ui, rect);

        let menu_area = rect.center(400.0, 600.0);
        let (title_area, content) = menu_area.split_top(200.0);

        Label::new("Lumina Tale")
            .size(60.0)
            .color(Color::WHITE)
            .align(Alignment::Center)
            .font("comforter")
            .show(ui, title_area);

        // 按钮区域布局
        let (btn_start, rest) = content.split_top(80.0);
        let (btn_settings, rest) = rest.split_top(80.0);
        let (btn_quit, _) = rest.split_top(80.0);

        let time = ui.time;

        let scale = 1.0 + (time * 3.0).sin() * 0.05;
        let rotation = (time * 2.0).sin() * 2.0;

        let start_rect = btn_start.shrink(10.0);
        let center_x = start_rect.x + start_rect.w / 2.0;
        let center_y = start_rect.y + start_rect.h / 2.0;

        let mut t = Transform::default();
        t.x = center_x;
        t.y = center_y;
        t.scale_x = scale;
        t.scale_y = scale;
        t.rotation = rotation;

        // 3. 绘制按钮 & 处理点击

        // --- 开始游戏 ---
        let mut start_clicked = false;

        ui.with_transform(t, &mut |ui| {
            let local_rect = Rect::new(
                -start_rect.w / 2.0,
                -start_rect.h / 2.0,
                start_rect.w,
                start_rect.h
            );

            if Button::new("Start Game")
                .rounded(8.0)
                .fill(Color::rgb(60, 100, 200))
                .show(ui, local_rect)
            {
                start_clicked = true;
            }
        });

        if start_clicked {
            *ctx = Ctx::default();
            let driver = ExecutorHandle::new(ctx, self.manager.clone());
            self.pending_transition = ScreenTransition::Replace(
                Box::new(InGameScreen::new(driver))
            );
        }

        if Button::new("Settings")
            .rounded(8.0)
            .show(ui, btn_settings.shrink(10.0))
        {
            self.pending_transition = ScreenTransition::Push(Box::new(SettingsScreen::new()));
        }

        if Button::new("Quit")
            .text_color(Color::rgb(255, 100, 100))
            .transparent() // 平时透明
            .stroke(Color::rgb(255, 100, 100), 1.0) // 红色边框
            .rounded(8.0)
            .show(ui, btn_quit.shrink(10.0))
        {
            self.pending_transition = ScreenTransition::Quit;
        }
    }
}