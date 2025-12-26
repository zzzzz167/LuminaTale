use serde::{Deserialize, Serialize};
use lumina_shared::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCfg {
    pub default_volume: f32,
    pub voice_volume:   f32,
    pub music_volume:   f32,
    pub music_looping: bool,
    pub fade_in: f32,
    pub fade_out:  f32,
    pub voice_link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugCfg {
    pub log_level: String,
    pub show_ast:  bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerCfg {
    pub trans_effect: String,
    pub scene_zindex: usize,
    pub sprite_zindex: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoreConfig {
    pub debug: DebugCfg,
    pub audio: AudioCfg,
    pub layer: LayerCfg,

    #[serde(default = "default_script_path")]
    pub script_path: String,
    pub ahead_step: usize,
}

fn default_script_path() -> String {
    "game/".to_string()
}

impl Default for CoreConfig {
    fn default() -> Self {
        CoreConfig {
            audio: AudioCfg {
                default_volume: 0.7,
                voice_volume: 0.7,
                voice_link: "_".to_string(),
                music_volume: 0.7,
                music_looping: true,
                fade_in: 0f32,
                fade_out: 0f32
            },
            debug: DebugCfg {
                log_level: "debug".to_string(),
                show_ast: false,
            },
            layer: LayerCfg {
                trans_effect: "dissolve".to_string(),
                scene_zindex: 0usize,
                sprite_zindex: 1usize,
            },
            script_path: default_script_path(),
            ahead_step: 20usize,
        }
    }
}

pub fn get() -> CoreConfig {
    config::get::<CoreConfig>("core")
}