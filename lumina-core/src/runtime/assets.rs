use std::collections::HashMap;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueRecord {
    pub speaker: Option<String>,
    pub text: String,
    pub voice_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Layers{
    pub arrange: Vec<String>,
    pub layer: HashMap<String, Vec<Sprite>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub target: String,
    pub attrs: Vec<String>,
    pub position: Option<String>,
    pub zindex: usize,
}