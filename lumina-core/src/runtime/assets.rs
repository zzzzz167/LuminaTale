use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub image_tag: Option<String>,
    pub voice_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audio {
    pub path: String,
    pub volume: f32,
    pub fade_in: f32,
    pub fade_out: f32,
    pub looping: bool,
}