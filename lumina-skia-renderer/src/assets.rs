use std::collections::HashMap;
use std::path::Path;
use std::fs;
use skia_safe::{Image, Data};

pub struct AssetManager {
    images: HashMap<String, Image>,
    root_path: String,
}

impl AssetManager {
    pub fn new(root_path: &str) -> Self {
        Self {
            images: HashMap::new(),
            root_path: root_path.to_string(),
        }
    }

    pub fn get_image(&mut self, name: &str) -> Option<Image> {
        if let Some(img) = self.images.get(name) {
            return Some(img.clone());
        }

        let path = Path::new(&self.root_path).join(format!("{}.png", name));

        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => {
                log::warn!("Asset not found: {:?}", path);
                return None;
            }
        };

        let data = Data::new_copy(&bytes);

        if let Some(image) = Image::from_encoded(data) {
            log::info!("Loaded asset: {}", name);
            self.images.insert(name.to_string(), image.clone());
            Some(image)
        } else {
            log::error!("Failed to decode image: {:?}", path);
            None
        }
    }
}