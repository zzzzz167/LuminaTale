use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub vsync: bool,
    pub assets: AssetsConfig,
    pub debug: DebugConfig
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsConfig {
    pub assets_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub skip_main_menu: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "LuminaTale (Skia)".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
            vsync: true, // 默认开启垂直同步，防止撕裂
            assets: AssetsConfig {
                assets_path: "./assets".to_string(),
            },
            debug: DebugConfig {
                skip_main_menu: false,
            }
        }
    }
}