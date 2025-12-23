use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use viviscript_core::ast::Script;

use super::{Screen, ScreenTransition};
// 引入 InGameScreen，以便跳转
use crate::screens::ingame::InGameScreen;
// use crate::screens::settings::SettingsScreen;

use crate::ui::UiDrawer;
use crate::core::{AssetManager, Painter, AudioPlayer};
use lumina_core::Ctx;
use lumina_core::renderer::driver::ExecutorHandle;

use lumina_ui::{Rect, Color};
use lumina_ui::widgets::{Button, Label, Panel};

pub struct MainMenuScreen {
    // 需要脚本引用来启动新游戏
    script: Arc<Script>,
    // 暂存这一帧 UI 点击产生的跳转指令
    pending_transition: ScreenTransition,
}

impl MainMenuScreen {
    pub fn new(script: Arc<Script>) -> Self {
        Self {
            script,
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
        _assets: &AssetManager,
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
        _assets: &mut AssetManager,
        rect: Rect,
        ctx: &mut Ctx
    ) {
        // 1. 绘制背景 (可以是纯色，也可以由 Painter 画一张背景图)
        // 这里简单用深色背景
        Panel::new()
            .color(Color::rgb(20, 20, 25))
            .show(ui, rect);

        // 2. 布局计算
        // 居中一个 400x600 的菜单区域
        let menu_area = rect.center(400.0, 600.0);

        // 顶部 200px 放标题
        let (title_area, content) = menu_area.split_top(200.0);

        Label::new("Lumina Tale")
            .size(60.0)
            .color(Color::WHITE)
            .show(ui, title_area);

        // 按钮区域布局
        let (btn_start, rest) = content.split_top(80.0);
        let (btn_settings, rest) = rest.split_top(80.0);
        let (btn_quit, _) = rest.split_top(80.0);

        // 3. 绘制按钮 & 处理点击

        // --- 开始游戏 ---
        if Button::new("Start Game").show(ui, btn_start.shrink(10.0)) {
            // A. 重置游戏上下文 (清空变量、立绘历史)
            *ctx = Ctx::default();

            // B. 创建执行驱动器
            let driver = ExecutorHandle::new(ctx, self.script.clone());

            // C. 设置跳转指令：替换当前屏幕为游戏屏幕
            self.pending_transition = ScreenTransition::Replace(
                Box::new(InGameScreen::new(driver))
            );
        }

        
        if Button::new("Settings").show(ui, btn_settings.shrink(10.0)) {
            // self.pending_transition = ScreenTransition::Push(Box::new(SettingsScreen::new()));
            println!("Settings clicked (TODO)");
        }

        // --- 退出 ---
        if Button::new("Quit")
            .text_color(Color::RED) // 红色文字警示
            .transparent()          // 透明背景
            .stroke(Color::RED, 1.0) // 红色边框
            .show(ui, btn_quit.shrink(10.0))
        {
            self.pending_transition = ScreenTransition::Quit;
        }
    }
}