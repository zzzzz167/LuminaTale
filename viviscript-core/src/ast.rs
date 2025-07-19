use crate::lexer::Span;

#[derive(Debug, PartialEq)]
pub struct Script {
    pub body: Vec<Stmt>
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    CharacterDef {
        span: Span,
        id: String,
        name: String,
        image_tag: Option<String>,
        voice_tag: Option<String>,
    },
    Label {
        span: Span,
        id: String,
        body: Vec<Stmt>
    },
    Choice {
        span: Span,
        title: Option<String>,
        arms: Vec<ChoiceArm>,
    },
    Jump {
        span: Span,
        target: String,
    },
    Call {
        span: Span,
        target: String,
    },
    LuaBlock {
        span: Span,
        code: String,
    },
    Dialogue {
        span: Span,
        speaker: Speaker,
        text: String,
        voice_index: Option<String>,
    },
    Narration {
        span: Span,
        lines: Vec<String>,
    },
    Audio {
        span: Span,
        action: AudioAction,
        channel: AudioChannel,
        resource: Option<String>, // None 时 action 必须是 Stop
        options: AudioOptions,
    },
    Hide {
        span: Span,
        target: String,
    },
    Show {
        span: Span,
        target: String,
        attrs: Option<Vec<ShowAttr>>, // 支持 +attr / -attr
        position: Option<String>,
        transition: Option<Transition>,
    },
    Scene {
        span: Span,
        image: Option<SceneImage>,
        transition: Option<Transition>
    },
    Error {
        span: Span,
        msg: String,
    },
}

#[derive(Debug, PartialEq)]
pub struct Speaker {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AudioAction {
    Play,
    Stop,
}

/// 音频通道
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AudioChannel {
    Music,
    Sound,
    Voice,
}

#[derive(Debug, PartialEq)]
pub struct AudioOptions {
    pub volume: Option<f32>,
    pub fade_in: Option<f32>,
    pub fade_out: Option<f32>,
    pub r#loop: bool,
}

#[derive(Debug, PartialEq)]
pub struct ChoiceArm {
    pub text: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
pub enum ShowAttr {
    Add(String),
    Remove(String),
}

#[derive(Debug, PartialEq)]
pub struct Transition {
    pub effect: String,
}

#[derive(Debug, PartialEq)]
pub struct SceneImage {
    pub prefix: String,
    pub attrs: Option<Vec<String>>
}
