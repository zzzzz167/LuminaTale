pub mod animator;
pub use animator::SceneAnimator;

use lumina_core::Ctx;
use lumina_core::renderer::driver::ExecutorHandle;
use crate::ui_state::UiState;

pub enum AppScene {
    MainMenu,
    InGame {
        ctx: Ctx,
        driver: ExecutorHandle,
        ui_state: UiState,
    }
}

impl Default for AppScene {
    fn default() -> Self {
        Self::MainMenu
    }
}