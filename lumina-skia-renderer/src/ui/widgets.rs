use skia_safe::{Rect, Point, Contains};

#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    None,
    Quit,
    OpenMenu(String),
    RunScript(String),
    ScriptChoice(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
}

pub struct Button {
    pub rect: Rect,
    pub text: String,
    pub action: UiAction,
    pub state: ButtonState,
}

impl Button {
    pub fn new(x: f32, y: f32, w: f32, h: f32, text: &str, action: UiAction) -> Self {
        Self {
            rect: Rect::new(x, y, x + w, y + h),
            text: text.to_string(),
            action,
            state: ButtonState::Normal,
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        self.rect.contains(p)
    }

    pub fn update_hover(&mut self, cursor: Point) {
        if self.rect.contains(cursor) {
            if self.state != ButtonState::Pressed {
                self.state = ButtonState::Hovered;
            }
        } else {
            self.state = ButtonState::Normal;
        }
    }
}
