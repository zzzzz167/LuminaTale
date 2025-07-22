#[derive(Debug, Clone)]
pub enum EngineEvent {
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

    ChoiceMade { index: usize },
    InputMode {mode: Mode},
}

#[derive(Debug, Clone)]
pub enum Mode {
    Continue,
    Exit
}