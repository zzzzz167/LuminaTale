#[derive(Debug, Clone)]
pub enum OutputEvent {
    ShowNarration { lines: Vec<String> },
    ShowDialogue { name: String, content: String },
    ShowChoice { title: Option<String>, options: Vec<String> },

    PlayAudio {channel: String, path: String, fade_in: f32, volume: f32 ,looping: bool},
    StopAudio {channel: String, fade_out: f32},


    NewScene {transition: String},
    NewSprite { transition: String },
    UpdateSprite { transition: String },
    HideSprite,

    StepDone,
    End,
    
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    ChoiceMade { index: usize },
    Continue,
    Exit,
    SaveRequest { slot: u32 },
    LoadRequest { slot: u32 },
}