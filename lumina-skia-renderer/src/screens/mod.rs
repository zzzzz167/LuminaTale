pub mod main_menu;
pub(crate) mod ingame;
pub mod settings;

use crate::ui::UiDrawer;
use crate::core::{AssetManager, AudioPlayer, Painter};
use lumina_core::Ctx;
use lumina_ui::Rect;
use winit::event_loop::ActiveEventLoop;

/// 屏幕切换指令
pub enum ScreenTransition {
    None,
    Push(Box<dyn Screen>),      // 打开新页面 (如设置)
    Pop,                        // 关闭当前页 (如关闭设置)
    Replace(Box<dyn Screen>),   // 彻底切换 (如 主菜单 -> 游戏)
    Quit,                       // 退出程序
}

/// 所有界面必须实现的 Trait
pub trait Screen {
    /// 逻辑更新 (返回切换指令)
    fn update(
        &mut self,
        dt: f32,
        ctx: &mut Ctx,
        el: &ActiveEventLoop,
        assets: &AssetManager,     // 新增
        audio: &mut AudioPlayer    // 新增
    ) -> ScreenTransition;

    /// 画面绘制
    fn draw(&mut self, ui: &mut UiDrawer, painter: &mut Painter, rect: Rect, ctx: &mut Ctx);
}