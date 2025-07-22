use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use crate::runtime::assets::{Audio, Character,DialogueRecord,Layers};
use crate::event::OutputEvent;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Ctx {
    pub characters: HashMap<String, Character>,
    pub audios: HashMap<String, Option<Audio>>,
    pub dialogue_history: Vec<DialogueRecord>,
    pub layer_record: Layers,
    #[serde(skip)]
    pub event_queue: VecDeque<OutputEvent>,
}

impl Ctx {
    pub fn push(&mut self, event: OutputEvent) {
        self.event_queue.push_back(event);
    }
    pub fn pop(&mut self) -> Option<OutputEvent> {
        self.event_queue.pop_front()
    }
    pub fn drain(&mut self) -> Vec<OutputEvent> {
        self.event_queue.drain(..).collect()
    }
}

