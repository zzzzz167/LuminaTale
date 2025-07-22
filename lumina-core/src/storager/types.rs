use crate::runtime::Ctx;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct FrameSnapshot {
    pub(crate) label: String,
    pub(crate) pc:    usize,
}

#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub ctx: Ctx,
    pub stack: Vec<FrameSnapshot>,
}