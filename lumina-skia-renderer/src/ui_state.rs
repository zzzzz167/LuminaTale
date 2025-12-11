use skia_safe::Rect;

#[derive(Debug, Clone, Default)]
pub enum UiMode {
    #[default]
    None,
    Choice {
        title: Option<String>,
        options: Vec<String>,
        hit_boxes: Vec<Rect>,
        hover_index: Option<usize>,
    }
}

#[derive(Debug, Default)]
pub struct UiState {
    pub mode: UiMode,
}

impl UiState {
    pub fn set_choices(&mut self, title: Option<String>, options: Vec<String>) {
        self.mode = UiMode::Choice {
            title,
            options,
            hit_boxes: vec![],
            hover_index: None,
        };
    }

    pub fn clear(&mut self) {
        self.mode = UiMode::None;
    }

    pub fn is_choosing(&self) -> bool {
        matches!(self.mode, UiMode::Choice { .. })
    }
}