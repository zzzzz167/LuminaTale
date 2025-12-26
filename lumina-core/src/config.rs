use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub assets_path: String, // 移到这里，Core也需要知道资源在哪
    pub script_path: String,
    pub save_path:   String, // ✅ 新增
    pub log_path:    String, // ✅ 新增
    pub log_level:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32, // 原 default_volume
    pub music_volume:  f32,
    pub voice_volume:  f32,
    pub sound_volume:  f32, // 补上 sound
    pub music_loop:    bool,
    pub fade_in_sec:   f32,
    pub fade_out_sec:  f32,
    pub voice_link_char: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub default_transition: String,
    pub preload_ahead: usize, // 原 ahead_step
    pub scene_zindex: usize,
    pub sprite_zindex: usize,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            assets_path: "assets/".into(),
            script_path: "game/".into(),
            save_path:   "saves/".into(),
            log_path:    "logs/".into(),
            log_level:   "info".into(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            voice_volume: 0.8,
            sound_volume: 0.8,
            music_loop: true,
            fade_in_sec: 0.2,
            fade_out_sec: 0.2,
            voice_link_char: "_".into(),
        }
    }
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            default_transition: "dissolve".into(),
            preload_ahead: 20,
            scene_zindex: 0,
            sprite_zindex: 10,
        }
    }
}