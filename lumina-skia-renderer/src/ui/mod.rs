use lumina_ui::input::{Interaction, UiContext};
use lumina_ui::{Color, Rect, UiRenderer};
use skia_safe::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextAlign, TextStyle};
use skia_safe::{Canvas, Paint, Point, Rect as SkRect};

pub struct UiDrawer<'a> {
    pub(crate) canvas: &'a Canvas,
    input: &'a UiContext,
    fonts: &'a FontCollection,
}

impl<'a> UiDrawer<'a> {
    pub fn new(canvas: &'a Canvas, input: &'a UiContext, fonts: &'a FontCollection) -> Self {
        Self { canvas, input, fonts }
    }

    fn to_skia_rect(&self, r: Rect) -> SkRect {
        SkRect::new(r.x, r.y, r.x + r.w, r.y + r.h)
    }

    fn to_skia_color(&self, c: Color) -> skia_safe::Color {
        skia_safe::Color::from_argb(c.a, c.r, c.g, c.b)
    }
}

impl <'a> UiRenderer for UiDrawer<'a> {
    fn draw_rect(&mut self, rect: Rect, color: Color) {
        let sk_rect = self.to_skia_rect(rect);
        let mut paint = Paint::default();
        paint.set_color(self.to_skia_color(color));
        paint.set_anti_alias(true);
        self.canvas.draw_round_rect(sk_rect, 4.0, 4.0, &paint);
    }

    fn draw_border(&mut self, rect: Rect, color: Color, width: f32) {
        let sk_rect = self.to_skia_rect(rect);
        let mut paint = Paint::default();
        paint.set_style(skia_safe::paint::Style::Stroke);
        paint.set_stroke_width(width);
        paint.set_color(self.to_skia_color(color));
        paint.set_anti_alias(true);
        self.canvas.draw_round_rect(sk_rect, 4.0, 4.0, &paint);
    }

    fn draw_text(&mut self, text: &str, rect: Rect, color: Color, size: f32) {
        let mut ts = TextStyle::new();
        ts.set_color(self.to_skia_color(color));
        ts.set_font_size(size);

        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&ts);
        ps.set_text_align(TextAlign::Center); // 默认居中，也可以传参控制

        let mut builder = ParagraphBuilder::new(&ps, self.fonts);
        builder.push_style(&ts);
        builder.add_text(text);

        let mut paragraph = builder.build();
        paragraph.layout(rect.w);

        let text_height = paragraph.height();
        let y = rect.y + (rect.h - text_height) / 2.0;

        paragraph.paint(self.canvas, Point::new(rect.x, y));
    }

    fn draw_circle(&mut self, center: (f32, f32), radius: f32, color: Color) {
        let mut paint = Paint::default();
        paint.set_color(self.to_skia_color(color));
        paint.set_anti_alias(true);

        self.canvas.draw_circle(Point::new(center.0, center.1), radius, &paint);
    }


    fn interact(&self, rect: Rect) -> Interaction {
        self.input.interact(rect)
    }

    fn cursor_pos(&self) -> (f32, f32) {
        self.input.mouse_pos
    }
}