use skia_safe::Rect;
use crate::ui::widgets::{Button, UiAction};

pub enum UiMode {
    None,
    Choice {
        title: Option<String>,
        buttons: Vec<Button>
    }
}

impl Default for UiMode {
    fn default() -> Self {
        UiMode::None
    }
}

#[derive(Default)]
pub struct UiState {
    pub mode: UiMode,
}

impl UiState {
    pub fn clear(&mut self) {
        self.mode = UiMode::None;
    }

    pub fn is_choosing(&self) -> bool {
        matches!(self.mode, UiMode::Choice { .. })
    }

    pub fn set_choices(&mut self, title: Option<String>, options: Vec<String>){
        let mut buttons = Vec::new();

        let btn_w = 500.0;
        let btn_h = 70.0;
        let gap = 20.0;
        let center_x = 1280.0 / 2.0;
        let total_h = options.len() as f32 * (btn_h + gap) - gap;
        let start_y = (720.0 - total_h) / 2.0;

        for (i, text) in options.into_iter().enumerate() {
            let y = start_y + i as f32 * (btn_h + gap);
            let x = center_x - btn_w / 2.0;

            // 关键：创建携带 ScriptChoice 动作的按钮
            let btn = Button::new(
                x, y, btn_w, btn_h,
                &text,
                UiAction::ScriptChoice(i)
            );
            buttons.push(btn);
        }

        self.mode = UiMode::Choice { title, buttons };
    }
}