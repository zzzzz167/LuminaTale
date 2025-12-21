use skia_safe::{Color, Paint, Rect};
use skia_safe::textlayout::{ParagraphBuilder, ParagraphStyle, TextStyle, TextAlign};
use crate::ui::{UiAction, RenderContext, WidgetRender};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
}

#[derive(Clone)]
pub struct Button {
    pub text: String,
    pub action: UiAction,
    pub state: ButtonState,
    pub rect: Rect,
}

impl Button {
    pub fn new(text: &str, action: UiAction) -> Self {
        Self {
            text: text.to_string(),
            action,
            state: ButtonState::Normal,
            rect: Rect::default(),
        }
    }
}

impl WidgetRender for Button {
    fn render(&self, ctx: &mut RenderContext) {
        let canvas = ctx.canvas;

        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let bg_color = match self.state {
            ButtonState::Normal => Color::from_rgb(60, 60, 60),
            ButtonState::Hovered => Color::from_rgb(80, 80, 100),
            ButtonState::Pressed => Color::from_rgb(40, 40, 40),
        };
        paint.set_color(bg_color);

        canvas.draw_round_rect(self.rect, 8.0, 8.0, &paint);

        if self.state == ButtonState::Hovered {
            let mut stroke = Paint::default();
            stroke.set_style(skia_safe::paint::Style::Stroke);
            stroke.set_stroke_width(2.0);
            stroke.set_color(Color::WHITE);
            stroke.set_anti_alias(true);
            canvas.draw_round_rect(self.rect, 8.0, 8.0, &stroke);
        }

        let mut ts = TextStyle::new();
        ts.set_color(Color::WHITE);
        ts.set_font_size(24.0);

        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&ts);
        ps.set_text_align(TextAlign::Center);

        let mut builder = ParagraphBuilder::new(&ps, ctx.fonts);
        builder.push_style(&ts);
        builder.add_text(&self.text);

        let mut paragraph = builder.build();
        paragraph.layout(self.rect.width());

        let text_h = paragraph.height();
        let y = self.rect.y() + (self.rect.height() - text_h) / 2.0;

        paragraph.paint(canvas, (self.rect.x(), y));
    }
}