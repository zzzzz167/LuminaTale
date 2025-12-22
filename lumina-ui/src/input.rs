use crate::Rect;

pub struct UiContext {
    /// 当前鼠标位置 (逻辑坐标)
    pub mouse_pos: (f32, f32),
    /// 鼠标左键是否刚刚按下 (本帧触发)
    pub mouse_pressed: bool,
    /// 鼠标左键是否处于按下状态 (拖拽用)
    pub mouse_held: bool,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            mouse_pos: (0.0, 0.0),
            mouse_pressed: false,
            mouse_held: false,
        }
    }

    /// 更新输入状态 (由 Renderer 调用)
    pub fn update(&mut self, x: f32, y: f32, pressed: bool, held: bool) {
        self.mouse_pos = (x, y);
        self.mouse_pressed = pressed;
        self.mouse_held = held;
    }

    pub fn interact(&self, rect: Rect) -> Interaction {
        let (mx, my) = self.mouse_pos;
        let hovered = rect.contains(mx, my);

        if hovered {
            if self.mouse_pressed {
                return Interaction::Clicked;
            }
            if self.mouse_held {
                return Interaction::Held;
            }
            return Interaction::Hovered;
        }

        Interaction::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interaction {
    None,
    Hovered,
    Clicked, // 刚刚点击
    Held,    // 按住中
}

impl Interaction {
    pub fn is_clicked(&self) -> bool {
        matches!(self, Interaction::Clicked)
    }
}