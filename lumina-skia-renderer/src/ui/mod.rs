use skia_safe::{Canvas, Color, Paint, Rect as SkRect, Point};
use skia_safe::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle, TextAlign};
use lumina_ui::{Rect, input::{UiContext, Interaction}};

pub struct UiDrawer<'a> {
    canvas: &'a Canvas,
    input: &'a UiContext,
    fonts: &'a FontCollection,
}

impl<'a> UiDrawer<'a> {
    pub fn new(canvas: &'a Canvas, input: &'a UiContext, fonts: &'a FontCollection) -> Self {
        Self { canvas, input, fonts }
    }

    fn to_skia(&self, r: Rect) -> SkRect {
        SkRect::new(r.x, r.y, r.x + r.w, r.y + r.h)
    }

    pub fn button(&mut self, text: &str, rect: Rect) -> bool {
        let interaction = self.input.interact(rect);

        let color = match interaction {
            Interaction::Clicked | Interaction::Held => Color::from_rgb(50, 50, 50),
            Interaction::Hovered => Color::from_rgb(80, 80, 100),
            Interaction::None => Color::from_rgb(40, 40, 40),
        };

        let sk_rect = self.to_skia(rect);

        // 绘制背景
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.set_anti_alias(true);
        self.canvas.draw_round_rect(sk_rect, 8.0, 8.0, &paint);

        // 绘制边框
        if interaction == Interaction::Hovered {
            let mut stroke = Paint::default();
            stroke.set_style(skia_safe::paint::Style::Stroke);
            stroke.set_stroke_width(2.0);
            stroke.set_color(Color::WHITE);
            stroke.set_anti_alias(true);
            self.canvas.draw_round_rect(sk_rect, 8.0, 8.0, &stroke);
        }

        let mut ts = TextStyle::new();
        ts.set_color(Color::WHITE);
        ts.set_font_size(24.0); // 字体大小

        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&ts);
        ps.set_text_align(TextAlign::Center); // 自动水平居中

        let mut builder = ParagraphBuilder::new(&ps, self.fonts);
        builder.push_style(&ts);
        builder.add_text(text);

        let mut paragraph = builder.build();
        paragraph.layout(rect.w); // 设置最大宽度

        // 计算垂直居中
        let text_height = paragraph.height();
        let y = rect.y + (rect.h - text_height) / 2.0;

        paragraph.paint(self.canvas, Point::new(rect.x, y));

        interaction.is_clicked()
    }

    pub fn label(&self, text: &str, rect: Rect, size: f32, color: Color) {
        let mut ts = TextStyle::new();
        ts.set_color(color);
        ts.set_font_size(size);

        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&ts);
        ps.set_text_align(TextAlign::Left); // 默认左对齐

        let mut builder = ParagraphBuilder::new(&ps, self.fonts);
        builder.push_style(&ts);
        builder.add_text(text);

        let mut paragraph = builder.build();
        paragraph.layout(rect.w);

        // 垂直居中
        let text_height = paragraph.height();
        let y = rect.y + (rect.h - text_height) / 2.0;

        paragraph.paint(self.canvas, Point::new(rect.x, y));
    }
}