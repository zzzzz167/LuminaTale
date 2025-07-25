use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use once_cell::sync::OnceCell;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub debug: DebugCfg,
    pub audio: AudioCfg,
    pub layer: LayerCfg,
}

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

impl Default for Config {
    fn default() -> Self {
        Config {
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
            }
        }
    }
}

static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn init_global<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    
    if !path.exists() {
        let default = Config::default();
        let toml = toml::to_string_pretty(&default).unwrap();
        fs::write(path, toml).unwrap();
        log::info!("Created default config at {:?}", path);
        CONFIG.set(default).ok();
        return;
    }
    
    let cfg: Config = toml::from_str(&fs::read_to_string(path).unwrap())
        .unwrap_or_else(|e| {
            log::warn!("Bad config {:?}: {} — using defaults", path, e);
            Config::default()
        });
    
    let toml = toml::to_string_pretty(&cfg).unwrap();
    fs::write(path, toml).unwrap();
    log::info!("Config upgraded & saved to {:?}", path);

    CONFIG.set(cfg).ok();
}

pub fn get() -> &'static Config {
    CONFIG.get().expect("config::init_global must be called first")
}