use skia_safe::{Rect, Color, Point};
use skia_safe::textlayout::{ParagraphBuilder, ParagraphStyle, TextStyle, TextAlign};
use crate::ui::{RenderContext, WidgetRender};

#[derive(Clone)]
pub struct Label {
    pub text: String,
    pub font_size: f32,
    pub color: Color,
    pub rect: Rect,
}

impl Label {
    pub fn new(text: &str, size: f32, color: Color) -> Self {
        Self {
            text: text.to_string(),
            font_size: size,
            color,
            rect: Rect::default(),
        }
    }
}

impl WidgetRender for Label {
    fn render(&self, ctx: &mut RenderContext) {
        let mut ts = TextStyle::new();
        ts.set_color(self.color);
        ts.set_font_size(self.font_size);

        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&ts);
        ps.set_text_align(TextAlign::Left); // 默认左对齐

        let mut builder = ParagraphBuilder::new(&ps, ctx.fonts);
        builder.push_style(&ts);
        builder.add_text(&self.text);

        let mut paragraph = builder.build();
        paragraph.layout(self.rect.width());

        // 垂直居中
        let text_h = paragraph.height();
        let y = self.rect.y() + (self.rect.height() - text_h) / 2.0;

        paragraph.paint(ctx.canvas, Point::new(self.rect.x(), y));
    }
}