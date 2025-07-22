use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use crate::runtime::assets::{Audio, Character};
use crate::event::EngineEvent;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Ctx {
    pub characters: HashMap<String, Character>,
    pub audios: HashMap<String, Option<Audio>>,
    pub dialogue_history: Vec<DialogueRecord>,
    pub layer_record: Layers,
    #[serde(skip)]
    pub event_queue: VecDeque<EngineEvent>,
}

impl Ctx {
    pub fn push(&mut self, event: EngineEvent) {
        self.event_queue.push_back(event);
    }
    pub fn pop(&mut self) -> Option<EngineEvent> {
        self.event_queue.pop_front()
    }
    pub fn drain(&mut self) -> Vec<EngineEvent> {
        self.event_queue.drain(..).collect()
    }
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