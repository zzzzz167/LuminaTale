use std::fs;
use std::path::Path;
use serde::Serialize;
use lumina_core::config::CoreConfig;

#[derive(Serialize)]
struct FullConfig {
    core: CoreConfig,

    // 只有开启 skia 时，才生成 window 配置节
    #[cfg(feature = "skia")]
    window: lumina_skia_renderer::config::WindowConfig,
}

pub fn ensure_config_exists(path: &str) {
    if Path::new(path).exists() {
        return;
    }

    println!("Creating default configuration at '{}'...", path);

    let default_config = FullConfig {
        core: CoreConfig::default(),

        #[cfg(feature = "skia")]
        window: lumina_skia_renderer::config::WindowConfig::default(),
    };

    let toml_str = toml::to_string_pretty(&default_config)
        .expect("Failed to serialize default config");

    if let Err(e) = fs::write(path, toml_str) {
        eprintln!("Failed to write config file: {}", e);
    } else {
        println!("Config file created successfully.");
    }
}