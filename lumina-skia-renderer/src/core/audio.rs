use std::collections::HashMap;
use std::time::Duration;
use kira::{AudioManager, DefaultBackend, AudioManagerSettings, sound::static_sound::{StaticSoundData, StaticSoundHandle}, Tween, Decibels, Value};
use kira::sound::FromFileError;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use log::{debug, error};
use crate::core::AssetManager;

enum AudioSource {
    Static(StaticSoundData),
    Streaming(StreamingSoundData<FromFileError>),
}

enum AudioHandle {
    Static(StaticSoundHandle),
    Streaming(StreamingSoundHandle<FromFileError>),
}

impl AudioHandle {
    // 辅助方法：统一设置音量
    fn set_volume(&mut self, volume: impl Into<Value<Decibels>>, tween: Tween) {
        match self {
            Self::Static(h) => { h.set_volume(volume, tween); },
            Self::Streaming(h) => { h.set_volume(volume, tween); },
        }
    }

    // 辅助方法：统一停止
    fn stop(&mut self, tween: Tween) {
        match self {
            Self::Static(h) => { h.stop(tween); },
            Self::Streaming(h) => { h.stop(tween); },
        }
    }
}

struct PendingPlay {
    channel: String,
    resource_id: String,
    volume: f32,
    fade_in_secs: f32,
    looping: bool,
    is_streaming: bool,
}

pub struct AudioPlayer{
    manager: AudioManager<DefaultBackend>,
    active_channels: HashMap<String, AudioHandle>,

    pending_queue: Vec<PendingPlay>,
}

impl AudioPlayer{
    pub fn new() -> Self{
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .expect("Failed to initialize Audio Manager");

        Self {
            manager,
            active_channels: HashMap::new(),
            pending_queue: Vec::new(),
        }
    }

    fn amplitude_to_db(amplitude: f32) -> Decibels {
        if amplitude <= 0.001 {
            Decibels::SILENCE
        } else {
            Decibels(20.0 * amplitude.log10())
        }
    }

    pub fn play(
        &mut self,
        assets: &mut AssetManager,
        channel: &str,
        resource_id: &str,
        volume: f32,
        fade_in_secs: f32,
        looping: bool
    ) {
        self.stop(channel, 0.1);

        let is_streaming = channel == "music" || channel == "bgm" || resource_id.starts_with("bgm_");

        let source = if is_streaming {
            // 注意：这里 assets.get_... 会把数据从缓存中 take() 走
            assets.get_streaming_audio(resource_id).map(AudioSource::Streaming)
        } else {
            assets.get_static_audio(resource_id).map(AudioSource::Static)
        };

        if let Some(audio_source) = source {
            self.play_internal(channel, audio_source, volume, fade_in_secs, looping);
        } else {
            // 没加载好，加入队列
            self.pending_queue.push(PendingPlay {
                channel: channel.to_string(),
                resource_id: resource_id.to_string(),
                volume,
                fade_in_secs,
                looping,
                is_streaming,
            });
        }
    }

    pub fn stop(&mut self, channel: &str, fade_out_secs: f32) {
        if let Some(mut handle) = self.active_channels.remove(channel) {
            let tween = if fade_out_secs > 0.0 {
                Tween { duration: Duration::from_secs_f32(fade_out_secs), ..Default::default() }
            } else { Tween::default() };
            handle.stop(tween);
        }

        self.pending_queue.retain(|p| p.channel != channel);
    }

    pub fn update(&mut self, assets: &mut AssetManager) {
        // 检查等待队列中的资源是否加载完毕
        if self.pending_queue.is_empty() { return; }

        let pending = std::mem::take(&mut self.pending_queue);

        for req in pending {
            let source = if req.is_streaming {
                assets.get_streaming_audio(&req.resource_id).map(AudioSource::Streaming)
            } else {
                assets.get_static_audio(&req.resource_id).map(AudioSource::Static)
            };

            if let Some(audio_source) = source {
                // 加载完成 -> 播放
                self.play_internal(
                    &req.channel,
                    audio_source,
                    req.volume,
                    req.fade_in_secs,
                    req.looping
                );
            } else {
                // 没好 -> 放回去
                self.pending_queue.push(req);
            }
        }
    }

    fn play_internal(&mut self, channel: &str, source: AudioSource, volume: f32, fade_in: f32, looping: bool) {
        let target_db = Self::amplitude_to_db(volume);

        let handle_result = match source {
            AudioSource::Static(mut d) => {
                if looping { d = d.loop_region(..); }
                if fade_in > 0.0 { d = d.volume(Decibels::SILENCE); }
                else { d = d.volume(target_db); }

                // 播放并包装成 Static 类型
                self.manager.play(d)
                    .map(AudioHandle::Static)
                    .map_err(|e| e.to_string())
            },
            AudioSource::Streaming(mut d) => {
                if looping { d = d.loop_region(..); }
                if fade_in > 0.0 { d = d.volume(Decibels::SILENCE); }
                else { d = d.volume(target_db); }

                // 播放并包装成 Streaming 类型
                self.manager.play(d)
                    .map(AudioHandle::Streaming)
                    .map_err(|e| e.to_string())
            },
        };

        match handle_result {
            Ok(mut handle) => {
                // 如果有淡入，现在统一设置
                if fade_in > 0.0 {
                    let tween = Tween {
                        duration: Duration::from_secs_f32(fade_in),
                        ..Default::default()
                    };
                    handle.set_volume(target_db, tween);
                }
                debug!("Audio playing: {}", channel);
                self.active_channels.insert(channel.to_string(), handle);
            },
            Err(e) => error!("Kira play error: {}", e),
        }
    }
}