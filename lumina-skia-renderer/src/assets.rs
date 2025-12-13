use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::time::{Duration, Instant};
use walkdir::WalkDir;
use skia_safe::{Image, Data};

pub struct AssetManager {
    cache: HashMap<String, (Image, Instant)>,
    path_index: HashMap<String, PathBuf>,
    root_path: PathBuf,
}

impl AssetManager {
    pub fn new(root_path: &str) -> Self {
        let mut manager = Self {
            cache: HashMap::new(),
            path_index: HashMap::new(),
            root_path: PathBuf::from(root_path),
        };

        manager.scan_assets();
        manager
    }

    fn scan_assets(&mut self) {
        log::info!("Scanning assets in {:?}...", self.root_path);

        for entry in WalkDir::new(&self.root_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext = ext.to_lowercase();
                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
                        // 获取文件名（不带后缀），作为 Key
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            // 检查重名冲突
                            if self.path_index.contains_key(stem) {
                                log::warn!("Duplicate asset name detected: '{}'. Overwriting with {:?}", stem, path);
                            }
                            self.path_index.insert(stem.to_string(), path.to_path_buf());
                        }
                    }
                }
            }
        }

        log::info!("Asset scan complete. Indexed {} files.", self.path_index.len());
    }

    pub fn get_image(&mut self, name: &str) -> Option<Image> {
        if let Some((img, last_used)) = self.cache.get_mut(name) {
            *last_used = Instant::now(); // 更新活跃时间
            return Some(img.clone());
        }

        let file_path = self.path_index.get(name)?;
        log::debug!("Loading asset: {} -> {:?}", name, file_path);
        let bytes = match fs::read(file_path) {
            Ok(b) => b,
            Err(e) => {
                log::error!("Failed to read file {:?}: {}", file_path, e);
                return None;
            }
        };

        let data = Data::new_copy(&bytes);
        if let Some(image) = Image::from_encoded(data) {
            // 存入缓存
            self.cache.insert(name.to_string(), (image.clone(), Instant::now()));
            Some(image)
        } else {
            log::error!("Failed to decode image: {:?}", file_path);
            None
        }
    }

    pub fn gc(&mut self, keep_alive: Duration){
        let now = Instant::now();
        let before_len = self.cache.len();

        self.cache.retain(|name, (_, last_used)| {
            let is_alive = now.duration_since(*last_used) < keep_alive;
            if !is_alive {
                log::debug!("GC: Unloading asset '{}'", name);
            }
            is_alive
        });

        let freed_count = before_len - self.cache.len();
        if freed_count > 0 {
            log::debug!("GC Triggered: Freed {} assets. Current cache size: {}", freed_count, self.cache.len());
        }
    }
}