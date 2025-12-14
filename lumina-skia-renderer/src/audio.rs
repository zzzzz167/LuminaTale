use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use kira::{
    AudioManager, DefaultBackend, AudioManagerSettings,
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
    Tween,
    Decibels
};
use log::{debug, error};

pub struct AudioPlayer{
    manager: AudioManager<DefaultBackend>,
    active_channels: HashMap<String, StaticSoundHandle>,
}

impl AudioPlayer{
    pub fn new() -> Self{
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .expect("Failed to initialize Audio Manager");

        Self {
            manager,
            active_channels: HashMap::new(),
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
        channel: &str,
        path: &PathBuf,
        volume: f32,
        fade_in_secs: f32,
        looping: bool
    ) {
        if let Some(mut old_handle) = self.active_channels.remove(channel) {
            let _ = old_handle.stop(Tween {
                duration: Duration::from_millis(100),
                ..Default::default()
            });
        }

       match StaticSoundData::from_file(path) {
           Ok(mut sound_data) => {
               if looping {
                   sound_data = sound_data.loop_region(..);
               }

               let target_db = Self::amplitude_to_db(volume);

               if fade_in_secs > 0.0 {
                   // 如果有淡入，初始音量设为静音，播放后再 Tween 到目标音量
                   sound_data = sound_data.volume(Decibels::SILENCE);
               } else {
                   sound_data = sound_data.volume(target_db);
               }

               match self.manager.play(sound_data) {
                   Ok(mut handle)=>{
                       if fade_in_secs > 0.0 {
                           let _ = handle.set_volume(
                               target_db,
                               Tween {
                                   duration: Duration::from_secs_f32(fade_in_secs),
                                   ..Default::default()
                               }
                           );
                       }

                       debug!("Audio started [{}]: {:?}", channel, path.file_name().unwrap_or_default());
                       self.active_channels.insert(channel.to_string(), handle);
                   },
                   Err(e) => error!("Kira play error: {}", e),
               }

           },
           Err(e) => error!("Failed to load audio {:?}: {}", path, e),
       }
    }

    pub fn stop(&mut self, channel: &str, fade_out_secs: f32) {
        if let Some(mut handle) = self.active_channels.remove(channel) {
            let tween = if fade_out_secs > 0.0 {
                Tween {
                    duration: Duration::from_secs_f32(fade_out_secs),
                    ..Default::default()
                }
            } else {
                Tween::default()
            };

            handle.stop(tween);

            debug!("Audio stopped [{}] (fade: {}s)", channel, fade_out_secs);
        }
    }
}