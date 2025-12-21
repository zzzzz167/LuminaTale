use skia_safe::{Rect, Color, Point, Contains};
use crate::ui::{UiAction, Direction, RenderContext, WidgetRender};
use crate::ui::widgets::{
    button::{Button, ButtonState},
    slider::Slider,
    checkbox::Checkbox,
    label::Label,
    image::Image,
};

#[derive(Debug, Clone)]
pub struct Style {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: f32,
    pub margin: f32,
    pub spacing: f32,
    pub bg_color: Option<Color>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            width: None, height: None,
            padding: 0.0, margin: 0.0, spacing: 10.0,
            bg_color: None,
        }
    }
}

pub enum WidgetNode {
    Container {
        children: Vec<WidgetNode>,
        direction: Direction,
        style: Style,
        computed_rect: Rect,
    },
    Button(Button),
    Slider(Slider),
    Checkbox(Checkbox),
    Label(Label),
    Image(Image),
    Spacer(f32),
}

impl WidgetNode {
    pub fn column(children: Vec<WidgetNode>) -> Self {
        WidgetNode::Container {
            children,
            direction: Direction::Column,
            style: Style::default(),
            computed_rect: Rect::default(),
        }
    }

    pub fn row(children: Vec<WidgetNode>) -> Self {
        WidgetNode::Container {
            children,
            direction: Direction::Row,
            style: Style::default(),
            computed_rect: Rect::default(),
        }
    }

    pub fn with_style(mut self, update: impl FnOnce(&mut Style)) -> Self {
        if let WidgetNode::Container { style, .. } = &mut self {
            update(style);
        }
        self
    }

    pub fn layout(&mut self, parent_rect: Rect) {
        match self {
            WidgetNode::Container { children, direction, style, computed_rect } => {
                let outer_rect = Rect::from_xywh(
                    parent_rect.x() + style.margin,
                    parent_rect.y() + style.margin,
                    style.width.unwrap_or(parent_rect.width() - style.margin * 2.0).max(0.0),
                    style.height.unwrap_or(parent_rect.height() - style.margin * 2.0).max(0.0),
                );
                *computed_rect = outer_rect;

                let (_, child_rects) = crate::ui::layout::compute_layout(
                    outer_rect, *direction, style.padding, style.spacing, children.len(),
                    |idx, max_w, max_h| {
                        let child = &children[idx];
                        (child.desired_width(max_w), child.desired_height(max_h))
                    }
                );

                for (child, rect) in children.iter_mut().zip(child_rects) {
                    child.layout(rect);
                }
            }
            WidgetNode::Button(b) => b.rect = parent_rect,
            WidgetNode::Slider(s) => s.rect = parent_rect,
            WidgetNode::Label(l) => l.rect = parent_rect,
            WidgetNode::Checkbox(c) => c.rect = parent_rect,
            WidgetNode::Image(i) => i.rect = parent_rect,
            WidgetNode::Spacer(_) => {},
        }
    }

    pub fn on_click(&mut self, pos: Point) -> UiAction {
        match self {
            WidgetNode::Container { children, .. } => {
                for child in children {
                    let action = child.on_click(pos);
                    if action != UiAction::None { return action; }
                }
                UiAction::None
            },
            WidgetNode::Button(b) => {
                if b.rect.contains(pos) {
                    b.state = ButtonState::Pressed;
                    return b.action.clone();
                }
                UiAction::None
            },
            WidgetNode::Checkbox(c) => {
                if c.rect.contains(pos) {
                    c.checked = !c.checked;
                    return c.on_change.clone();
                }
                UiAction::None
            },
            WidgetNode::Slider(s) => {
                if s.rect.contains(pos) {
                    s.is_dragging = true;
                    s.update_drag(pos.x);
                    return UiAction::AdjustVolume(s.config_key, s.value);
                }
                UiAction::None
            },
            _ => UiAction::None,
        }
    }

    pub fn on_mouse_move(&mut self, pos: Point) -> UiAction {
        match self {
            WidgetNode::Container { children, .. } => {
                for child in children { child.on_mouse_move(pos); }
                UiAction::None
            },
            WidgetNode::Button(b) => {
                if b.rect.contains(pos) {
                    if b.state != ButtonState::Pressed { b.state = ButtonState::Hovered; }
                } else {
                    b.state = ButtonState::Normal;
                }
                UiAction::None
            },
            WidgetNode::Slider(s) => {
                if s.is_dragging {
                    s.update_drag(pos.x);
                    return UiAction::AdjustVolume(s.config_key, s.value);
                }
                UiAction::None
            },
            _ => UiAction::None
        }
    }

    pub fn on_mouse_release(&mut self) {
        match self {
            WidgetNode::Container { children, .. } => {
                for child in children { child.on_mouse_release(); }
            },
            WidgetNode::Button(b) => {
                if b.state == ButtonState::Pressed { b.state = ButtonState::Hovered; }
            },
            WidgetNode::Slider(s) => s.is_dragging = false,
            _ => {}
        }
    }

    fn desired_width(&self, max: f32) -> f32 {
        match self {
            WidgetNode::Container { style, .. } => style.width.unwrap_or(max),
            WidgetNode::Spacer(v) => *v,
            _ => max,
        }
    }

    fn desired_height(&self, max: f32) -> f32 {
        match self {
            WidgetNode::Container { style, .. } => style.height.unwrap_or(max),
            WidgetNode::Button(_) => 60.0,
            WidgetNode::Slider(_) => 40.0,
            WidgetNode::Label(l) => l.font_size + 10.0,
            WidgetNode::Checkbox(_) => 30.0,
            WidgetNode::Image(_) => 200.0,
            WidgetNode::Spacer(v) => *v,
        }
    }
}

impl WidgetRender for WidgetNode {
    fn render(&self, ctx: &mut RenderContext) {
        match self {
            // 容器：画自己背景 -> 递归画子元素
            WidgetNode::Container { children, style, computed_rect, .. } => {
                if let Some(color) = style.bg_color {
                    let mut paint = skia_safe::Paint::default();
                    paint.set_color(color);
                    ctx.canvas.draw_rect(*computed_rect, &paint);
                }

                for child in children {
                    child.render(ctx); // 👈 递归调用！
                }
            },
            WidgetNode::Button(b) => b.render(ctx),
            WidgetNode::Slider(s) => s.render(ctx),
            WidgetNode::Checkbox(c) => c.render(ctx),
            WidgetNode::Label(l) => l.render(ctx),
            WidgetNode::Image(i) => i.render(ctx),
            WidgetNode::Spacer(_) => {},
        }
    }
}