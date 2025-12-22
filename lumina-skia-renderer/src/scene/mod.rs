pub mod animator;
pub use animator::SceneAnimator;

use lumina_core::renderer::driver::ExecutorHandle;
use lumina_core::Ctx;

pub enum AppScene {
    MainMenu,

    InGame {
        ctx: Ctx,
        driver: ExecutorHandle,
    },
    Settings {
        prev_scene: Box<AppScene>,
    }
}

impl Default for AppScene {
    fn default() -> Self {
        AppScene::MainMenu
    }
}

impl AppScene {
    pub fn new_settings(prev: AppScene) -> Self {
        AppScene::Settings {
            prev_scene: Box::new(prev),
        }
    }
}