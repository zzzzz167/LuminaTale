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

    #[serde(default)]
    #[serde(with = "json_as_string")]
    pub var_f: serde_json::Value,

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

mod json_as_string {
    use super::*;
    use serde::de::Error as DeError;
    use serde::ser::Error as SerError;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 把 JSON Value 转成 String
        let s = serde_json::to_string(value).map_err(S::Error::custom)?;
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 把 String 读出来，再转回 JSON Value
        let s = String::deserialize(deserializer)?;
        serde_json::from_str(&s).map_err(D::Error::custom)
    }
}