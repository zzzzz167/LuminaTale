use skia_safe::{Color, Rect};
use crate::ui::{
    WidgetNode,
    Button,
    UiAction,
    Style,
    Direction,
    Label
};
#[derive(Default)]
pub struct UiState {
    pub mode: UiMode,
}

pub enum UiMode {
    None,
    Choice {
        title: Option<String>,
        root: WidgetNode,
    },
}

impl Default for UiMode {
    fn default() -> Self {
        UiMode::None
    }
}

impl UiState {
    pub fn clear(&mut self) {
        self.mode = UiMode::None;
    }

    pub fn is_choosing(&self) -> bool {
        matches!(self.mode, UiMode::Choice { .. })
    }

    /// 将传入的选项字符串转换为 Widget 树
    pub fn set_choices(&mut self, title: Option<String>, options: Vec<String>) {
        let mut children = Vec::new();

        // 1. 如果有标题，先加个 Label
        if let Some(t) = &title {
            children.push(WidgetNode::Label(Label::new(t, 30.0, Color::WHITE)));
            children.push(WidgetNode::Spacer(20.0));
        }

        // 2. 构建选项按钮
        for (i, text) in options.into_iter().enumerate() {
            let btn = Button::new(&text, UiAction::ScriptChoice(i));
            children.push(WidgetNode::Button(btn));
        }

        // 3. 构建容器 (VBox)
        let mut root = WidgetNode::Container {
            direction: Direction::Column,
            children,
            style: Style {
                width: Some(600.0), // 限制宽度
                padding: 40.0,
                margin: 0.0,
                spacing: 15.0,
                bg_color: Some(Color::from_argb(200, 0, 0, 0)), // 半透明黑背景
                ..Default::default()
            },
            computed_rect: Rect::default(), // 等待 layout
        };

        // 4. 预计算布局 (假设屏幕 1280x720)
        let screen_w = 1280.0;
        let screen_h = 720.0;

        let menu_w = 600.0;
        let menu_h = 500.0;

        let x = (screen_w - menu_w) / 2.0;
        let y = (screen_h - menu_h) / 2.0;

        // 手动触发布局计算
        root.layout(Rect::from_xywh(x, y, menu_w, menu_h));

        self.mode = UiMode::Choice { title, root };
    }
}