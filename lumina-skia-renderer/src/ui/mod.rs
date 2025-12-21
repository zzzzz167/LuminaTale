//TODO: 设计更合理和先进的布局系统

pub mod widgets;
pub mod layout;
pub mod node;
pub mod render;
pub use widgets::{button::Button, checkbox::Checkbox, image::Image, label::Label, slider::Slider};
pub use node::{WidgetNode, Style};
pub use render::{RenderContext, WidgetRender};

#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    None,
    Quit,
    Back,
    OpenMenu(String),
    RunScript(String),
    ScriptChoice(usize),
    AdjustVolume(&'static str, f32),
    ToggleConfig(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Column,
    Row,
}