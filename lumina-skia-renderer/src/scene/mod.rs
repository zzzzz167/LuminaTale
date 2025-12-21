pub mod animator;
pub use animator::SceneAnimator;

use lumina_core::Ctx;
use lumina_core::renderer::driver::ExecutorHandle;
use crate::ui::UiState;
use crate::ui::widgets::{Button, UiAction};

pub enum AppScene {
    MainMenu {
        buttons: Vec<Button>,
    },
    InGame {
        ctx: Ctx,
        driver: ExecutorHandle,
        ui_state: UiState,
    }
}

impl Default for AppScene {
    fn default() -> Self {
        // 初始化默认的主菜单布局
        let btn_w = 200.0;
        let btn_h = 60.0;
        let start_x = (1280.0 - btn_w) / 2.0;
        let start_y = 300.0;
        let gap = 20.0;

        // 使用通用 Action 定义行为
        let btn_start = Button::new(
            start_x, start_y, btn_w, btn_h,
            "Start Game",
            UiAction::RunScript("StartGame".to_string())
        );

        let btn_quit = Button::new(
            start_x, start_y + btn_h + gap, btn_w, btn_h,
            "Quit",
            UiAction::Quit
        );

        AppScene::MainMenu {
            buttons: vec![btn_start, btn_quit],
        }
    }
}