use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use walkdir::WalkDir;
use skia_safe::{Image, Data, FontMgr};
use skia_safe::textlayout::TypefaceFontProvider;
use kira::sound::static_sound::StaticSoundData;
use kira::sound::streaming::StreamingSoundData;
use crossbeam_channel::{unbounded, Receiver, Sender};
use kira::sound::FromFileError;
use log::{debug, error, info, warn};

#[derive(Clone)]
pub enum AssetData {
    Image(Image),
    StaticAudio(StaticSoundData),
    StreamingAudio(Arc<Mutex<Option<StreamingSoundData<FromFileError>>>>),
}

#[derive(Clone)]
enum AssetState {
    Loading,
    Ready(AssetData, Instant),
    Failed(String),
}

enum LoadRequest {
    LoadImage { id: String, path: PathBuf },
    LoadStaticAudio { id: String, path: PathBuf },
    LoadStreamingAudio { id: String, path: PathBuf },
}

enum LoadResult {
    ImageBytes { id: String, data: Data },
    StaticAudioData { id: String, data: StaticSoundData },
    StreamingAudioData { id: String, data: StreamingSoundData<FromFileError> },
    Error { id: String, msg: String },
}



pub struct AssetManager {
    root_path: PathBuf,
    image_paths: HashMap<String, PathBuf>,
    audio_paths: HashMap<String, PathBuf>,
    font_paths: HashMap<String, PathBuf>,

    cache: HashMap<String, AssetState>,

    tx_request: Sender<LoadRequest>,
    rx_result: Receiver<LoadResult>,
}

impl AssetManager {
    pub fn new(root_path: &str) -> Self {
        let (tx_request, rx_request) = unbounded::<LoadRequest>();
        let (tx_result, rx_result) = unbounded::<LoadResult>();
        let tx_res_worker = tx_result.clone();

        thread::Builder::new()
            .name("AssetWorker".into())
            .spawn(move || {
                info!("AssetWorker started");
                while let Ok(req) = rx_request.recv() {
                    match req {
                        LoadRequest::LoadImage { id, path } => {
                            match fs::read(&path) {
                                Ok(bytes) => {
                                    let data = Data::new_copy(&bytes);
                                    let _ = tx_result.send(LoadResult::ImageBytes { id, data });
                                }
                                Err(e) => {
                                    let _ = tx_result.send(LoadResult::Error { id, msg: e.to_string() });
                                }
                            }
                        },
                        LoadRequest::LoadStaticAudio { id, path } => {
                            match StaticSoundData::from_file(&path) {
                                Ok(data) => {
                                    let _ = tx_res_worker.send(LoadResult::StaticAudioData { id, data });
                                }
                                Err(e) => {
                                    let _ = tx_res_worker.send(LoadResult::Error { id, msg: e.to_string() });
                                }
                            }
                        },
                        LoadRequest::LoadStreamingAudio { id, path } => {
                            match StreamingSoundData::from_file(&path) {
                                Ok(data) => {
                                    let _ = tx_res_worker.send(LoadResult::StreamingAudioData { id, data });
                                }
                                Err(e) => {
                                    let _ = tx_res_worker.send(LoadResult::Error { id, msg: e.to_string() });
                                }
                            }
                        }
                    }
                }
            }).expect("Failed to spawn asset worker");

        let mut manager = Self {
            root_path: PathBuf::from(root_path),
            image_paths: HashMap::new(),
            audio_paths: HashMap::new(),
            font_paths: HashMap::new(),
            cache: HashMap::new(),
            tx_request,
            rx_result,
        };

        manager.scan_assets();
        manager
    }

    fn scan_assets(&mut self) {
        info!("Scanning assets in {:?}...", self.root_path);

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
                            "ttf" | "otf" | "ttc" => {
                                self.font_paths.insert(key, path.to_path_buf());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        info!("Asset scan complete. Images: {}, Audio: {}, Font: {}",
            self.image_paths.len(), self.audio_paths.len(), self.font_paths.len());
    }

    pub fn gc(&mut self, keep_alive: Duration) {
        let now = Instant::now();
        self.cache.retain(|_, state| {
            match state {
                AssetState::Ready(_, last_used) => {
                    now.duration_since(*last_used) < keep_alive
                },
                _ => true
            }
        });
    }

    pub fn get_image(&mut self, name: &str) -> Option<Image> {
        if let Some(state) = self.cache.get_mut(name) {
            return match state {
                AssetState::Ready(AssetData::Image(img), last_used) => {
                    *last_used = Instant::now();
                    Some(img.clone())
                },
                _ => None,
            }
        }
        if let Some(path) = self.image_paths.get(name).cloned() {
            self.cache.insert(name.to_string(), AssetState::Loading);

            let _ = self.tx_request.send(LoadRequest::LoadImage {
                id: name.to_string(),
                path
            });
            debug!("Async load requested: [Image] {}", name);
        } else {
            warn!("Image not found in index: {}", name);
            self.cache.insert(name.to_string(), AssetState::Failed("File not found".into()));
        }

        None
    }

    pub fn get_static_audio(&mut self, name: &str) -> Option<StaticSoundData> {
        if let Some(state) = self.cache.get_mut(name) {
            return match state {
                AssetState::Ready(AssetData::StaticAudio(data), last_used) => {
                    *last_used = Instant::now();
                    Some(data.clone())
                },
                _ => None,
            }
        }
        if let Some(path) = self.audio_paths.get(name).cloned() {
            self.cache.insert(name.to_string(), AssetState::Loading);
            let _ = self.tx_request.send(LoadRequest::LoadStaticAudio { id: name.to_string(), path });
        }
        None
    }

    pub fn get_streaming_audio(&mut self, name: &str) -> Option<StreamingSoundData<FromFileError>> {
        if let Some(state) = self.cache.get_mut(name) {
            match state {
                AssetState::Ready(AssetData::StreamingAudio(arc_mutex), last_used) => {
                    *last_used = Instant::now();
                    let mut guard = arc_mutex.lock().unwrap();
                    if let Some(data) = guard.take() {
                        return Some(data);
                    }
                },
                _ => return None,
            }
        }
        if let Some(path) = self.audio_paths.get(name).cloned() {
            self.cache.insert(name.to_string(), AssetState::Loading);
            let _ = self.tx_request.send(LoadRequest::LoadStreamingAudio { id: name.to_string(), path });
        }
        None
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
                        info!("Registered font: '{}'", name);
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

    pub fn update(&mut self) {
        while let Ok(result) = self.rx_result.try_recv() {
            match result {
                LoadResult::ImageBytes { id, data } => {
                    if let Some(img) = Image::from_encoded(data) {
                        self.cache.insert(id, AssetState::Ready(AssetData::Image(img), Instant::now()));
                    } else {
                        self.cache.insert(id, AssetState::Failed("Decode failed".into()));
                    }
                },
                LoadResult::StaticAudioData { id, data } => {
                    self.cache.insert(id, AssetState::Ready(AssetData::StaticAudio(data), Instant::now()));
                },
                LoadResult::StreamingAudioData { id, data } => {
                    let wrapper = Arc::new(Mutex::new(Some(data)));
                    self.cache.insert(id, AssetState::Ready(AssetData::StreamingAudio(wrapper), Instant::now()));
                },
                LoadResult::Error { id, msg } => {
                    error!("Load Error [{}]: {}", id, msg);
                    self.cache.insert(id, AssetState::Failed(msg));
                }
            }
        }
    }
}