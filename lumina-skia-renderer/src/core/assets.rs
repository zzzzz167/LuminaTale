use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::time::{Duration, Instant};
use walkdir::WalkDir;
use skia_safe::{Image, Data, FontMgr};
use skia_safe::textlayout::TypefaceFontProvider;

pub struct AssetManager {
    image_cache: HashMap<String, (Image, Instant)>,
    image_paths: HashMap<String, PathBuf>,
    audio_paths: HashMap<String, PathBuf>,
    font_paths: HashMap<String, PathBuf>,
    root_path: PathBuf,
}

impl AssetManager {
    pub fn new(root_path: &str) -> Self {
        let mut manager = Self {
            image_cache: HashMap::new(),
            image_paths: HashMap::new(),
            audio_paths: HashMap::new(),
            font_paths: HashMap::new(),
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
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let key = stem.to_string();

                        match ext.as_str() {
                            "png" | "jpg" | "jpeg" => {
                                self.image_paths.insert(key, path.to_path_buf());
                            },
                            "mp3" | "wav" | "ogg" | "flac" => {
                                self.audio_paths.insert(key, path.to_path_buf());
                            },
                            "ttf" | "otf" => {
                                self.font_paths.insert(key, path.to_path_buf());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        log::info!("Asset scan complete. Images: {}, Audio: {}, Font: {}",
            self.image_paths.len(), self.audio_paths.len(), self.font_paths.len());
    }

    pub fn get_image(&mut self, name: &str) -> Option<Image> {
        if let Some((img, last_used)) = self.image_cache.get_mut(name) {
            *last_used = Instant::now(); // 更新活跃时间
            return Some(img.clone());
        }

        let file_path = self.image_paths.get(name)?;
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
            self.image_cache.insert(name.to_string(), (image.clone(), Instant::now()));
            Some(image)
        } else {
            log::error!("Failed to decode image: {:?}", file_path);
            None
        }
    }

    pub fn get_audio_path(&self, name: &str) -> Option<&PathBuf> {
        self.audio_paths.get(name)
    }

    pub fn gc(&mut self, keep_alive: Duration){
        let now = Instant::now();
        let before_len = self.image_cache.len();

        self.image_cache.retain(|name, (_, last_used)| {
            let is_alive = now.duration_since(*last_used) < keep_alive;
            if !is_alive {
                log::debug!("GC: Unloading asset '{}'", name);
            }
            is_alive
        });

        let freed_count = before_len - self.image_cache.len();
        if freed_count > 0 {
            log::debug!("GC Triggered: Freed {} assets. Current cache size: {}", freed_count, self.image_cache.len());
        }
    }

    pub fn register_fonts_to(&self, provider: &mut TypefaceFontProvider) {
        for (name, path) in &self.font_paths {
            // 读取文件字节
            match fs::read(path) {
                Ok(bytes) => {
                    let data = Data::new_copy(&bytes);
                    // 创建 Typeface
                    if let Some(typeface) = FontMgr::default().new_from_data(&data, None) {
                        // 注册！使用文件名作为 alias (别名)
                        provider.register_typeface(typeface, Some(name.as_str()));
                        log::info!("Registered font: '{}'", name);
                    } else {
                        log::error!("Failed to parse font: {:?}", path);
                    }
                },
                Err(e) => {
                    log::error!("Failed to read font file {:?}: {}", path, e);
                }
            }
        }
    }
}