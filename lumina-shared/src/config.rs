use std::sync::RwLock;
use std::path::Path;
use std::fs;
use once_cell::sync::OnceCell;
use serde::de::DeserializeOwned;
use toml::Table;

static GLOBAL_CONFIG: OnceCell<RwLock<Table>> = OnceCell::new();

pub fn init<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let path = path.as_ref();

    let content = if path.exists() {
        log::info!("Loading config from {:?}", path);
        fs::read_to_string(path)?
    } else {
        log::warn!("Config file not found at {:?}, using defaults.", path);
        String::new()
    };

    let table: Table = toml::from_str(&content).unwrap_or_else(|e| {
        log::error!("Config syntax error: {}, using empty config.", e);
        Table::new()
    });

    GLOBAL_CONFIG.set(RwLock::new(table))
        .map_err(|_| anyhow::anyhow!("Config already initialized"))?;

    Ok(())
}

pub fn get<T: DeserializeOwned + Default>(key: &str) -> T {
    let store = GLOBAL_CONFIG.get().expect("lumina-shared config not initialized!");
    let read_guard = store.read().unwrap();

    if let Some(value) = read_guard.get(key) {
        value.clone().try_into().unwrap_or_else(|e| {
            log::warn!("Config section '[{}]' mismatch: {}. Using default.", key, e);
            T::default()
        })
    } else {
        T::default()
    }
}