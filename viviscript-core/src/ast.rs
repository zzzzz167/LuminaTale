//! High-level abstract syntax tree (AST) for the visual-novel scripting language.
//!
//! All AST nodes carry a [`Span`] that identifies the source location they were
//! parsed from.

use crate::lexer::Span;

/// The root node of every compiled script.
#[derive(Debug, PartialEq)]
pub struct Script {
    pub body: Vec<Stmt>
}

/// A single statement in the visual-novel DSL.
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    /// Declares a character that can later appear in dialogue or other commands.
    CharacterDef {
        span: Span,
        id: String,
        name: String,
        image_tag: Option<String>,
        voice_tag: Option<String>,
    },
    /// Defines a label that can be jumped to or called.
    Label {
        span: Span,
        id: String,
        body: Vec<Stmt>
    },
    /// Displays a menu of choices to the player.
    Choice {
        span: Span,
        title: Option<String>,
        arms: Vec<ChoiceArm>,
    },
    /// Unconditional jump to another label.
    Jump {
        span: Span,
        target: String,
    },
    /// Calls a label as a subroutine, returning afterward.
    Call {
        span: Span,
        target: String,
    },
    /// Inline Lua code block executed at runtime.
    LuaBlock {
        span: Span,
        code: String,
    },
    /// A line of dialogue spoken by a character.
    Dialogue {
        span: Span,
        speaker: Speaker,
        text: String,
        voice_index: Option<String>,
    },
    /// Narration or internal monologue that does not belong to any character.
    Narration {
        span: Span,
        lines: Vec<String>,
    },
    /// Controls audio playback on a specific channel.
    Audio {
        span: Span,
        action: AudioAction,
        channel: String,
        resource: Option<String>, // None 时 action 必须是 Stop
        options: AudioOptions,
    },
    /// Removes a previously shown image or sprite from the screen.
    Hide {
        span: Span,
        target: String,
    },
    /// Displays or updates an image or sprite.
    Show {
        span: Span,
        target: String,
        attrs: Option<Vec<ShowAttr>>, // 支持 +attr / -attr
        position: Option<String>,
        transition: Option<Transition>,
    },
    /// Replaces the entire background or scene image.
    Scene {
        span: Span,
        image: Option<SceneImage>,
        transition: Option<Transition>
    },
    /// Placeholder node emitted when the parser encounters a syntax error.
    Error {
        span: Span,
        msg: String,
    },
}

/// Identifies the speaker of a dialogue line.
#[derive(Debug, PartialEq, Clone)]
pub struct Speaker {
    pub name: String,
    pub alias: Option<String>,
}

/// Available audio actions.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AudioAction {
    Play,
    Stop,
}

/// Fine-grained configuration for an audio command.
#[derive(Debug, PartialEq, Clone)]
pub struct AudioOptions {
    pub volume: Option<f32>,
    pub fade_in: Option<f32>,
    pub fade_out: Option<f32>,
    pub r#loop: bool,
}

/// A single selectable option inside a `Choice`.
#[derive(Debug, PartialEq, Clone)]
pub struct ChoiceArm {
    pub text: String,
    pub body: Vec<Stmt>,
}

/// Attribute modification for use in `Show`.
#[derive(Debug, PartialEq, Clone)]
pub enum ShowAttr {
    Add(String),
    Remove(String),
}

/// Transition effect applied when changing visuals.
#[derive(Debug, PartialEq, Clone)]
pub struct Transition {
    pub effect: String,
}

/// Configuration for a scene image.
#[derive(Debug, PartialEq, Clone)]
pub struct SceneImage {
    pub prefix: String,
    pub attrs: Option<Vec<String>>
}
