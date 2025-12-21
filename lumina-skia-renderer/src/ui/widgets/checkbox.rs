use skia_safe::{Rect, Color, Paint, Point};
use skia_safe::textlayout::{ParagraphBuilder, ParagraphStyle, TextStyle, TextAlign};
use crate::ui::{UiAction, RenderContext, WidgetRender};

#[derive(Clone)]
pub struct Checkbox {
    pub label: String,
    pub checked: bool,
    pub on_change: UiAction,
    pub rect: Rect,
}

impl Checkbox {
    pub fn new(label: &str, checked: bool, action: UiAction) -> Self {
        Self {
            label: label.to_string(),
            checked,
            on_change: action,
            rect: Rect::default(),
        }
    }
}

impl WidgetRender for Checkbox {
    fn render(&self, ctx: &mut RenderContext) {
        let canvas = ctx.canvas;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let box_size = 24.0;
        let cy = self.rect.center_y();
        let box_rect = Rect::from_xywh(self.rect.left, cy - box_size/2.0, box_size, box_size);

        paint.set_color(Color::WHITE);
        paint.set_style(skia_safe::paint::Style::Stroke);
        paint.set_stroke_width(2.0);
        canvas.draw_rect(box_rect, &paint);

        if self.checked {
            paint.set_style(skia_safe::paint::Style::Fill);
            paint.set_color(Color::from_rgb(100, 200, 255));
            let inner = box_rect.with_inset((5.0, 5.0));
            canvas.draw_rect(inner, &paint);
        }

        let mut ts = TextStyle::new();
        ts.set_color(Color::WHITE);
        ts.set_font_size(20.0);

        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&ts);
        ps.set_text_align(TextAlign::Left);

        let mut builder = ParagraphBuilder::new(&ps, ctx.fonts);
        builder.push_style(&ts);
        builder.add_text(&self.label);

        let mut paragraph = builder.build();
        let text_w = (self.rect.width() - box_size - 10.0).max(0.0);
        paragraph.layout(text_w);

        // 垂直居中
        let text_h = paragraph.height();
        let text_y = self.rect.top() + (self.rect.height() - text_h) / 2.0;

        paragraph.paint(canvas, Point::new(box_rect.right + 10.0, text_y));
    }
}