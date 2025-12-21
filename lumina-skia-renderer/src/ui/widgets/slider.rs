use skia_safe::{Rect, Color, Paint, Point};
use skia_safe::textlayout::{ParagraphBuilder, ParagraphStyle, TextStyle, TextAlign};
use crate::ui::{RenderContext, WidgetRender};

#[derive(Clone)]
pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub label: String,
    pub config_key: &'static str,
    pub is_dragging: bool,
    pub rect: Rect,
}

impl Slider {
    pub fn new(label: &str, key: &'static str, val: f32) -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            value: val.clamp(0.0, 1.0),
            label: label.to_string(),
            config_key: key,
            is_dragging: false,
            rect: Rect::default(),
        }
    }

    pub fn update_drag(&mut self, mouse_x: f32) -> f32 {
        let width = self.rect.width();
        if width <= 0.0 { return self.value; }

        let relative_x = mouse_x - self.rect.left;
        let percentage = (relative_x / width).clamp(0.0, 1.0);
        self.value = self.min + percentage * (self.max - self.min);
        self.value
    }
}

impl WidgetRender for Slider {
    fn render(&self, ctx: &mut RenderContext) {
        let canvas = ctx.canvas;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let cy = self.rect.center_y();
        let track_h = 6.0;

        paint.set_color(Color::from_rgb(50, 50, 50));
        let track_rect = Rect::from_xywh(
            self.rect.left, cy - track_h/2.0,
            self.rect.width(), track_h
        );
        canvas.draw_round_rect(track_rect, 3.0, 3.0, &paint);

        let fill_w = self.rect.width() * ((self.value - self.min) / (self.max - self.min));
        if fill_w > 0.0 {
            paint.set_color(Color::from_rgb(100, 180, 255));
            let fill_rect = Rect::from_xywh(self.rect.left, cy - track_h/2.0, fill_w, track_h);
            canvas.draw_round_rect(fill_rect, 3.0, 3.0, &paint);
        }

        let handle_x = self.rect.left + fill_w;
        paint.set_color(if self.is_dragging { Color::LIGHT_GRAY } else { Color::WHITE });
        canvas.draw_circle(Point::new(handle_x, cy), 10.0, &paint);

        let label_text = format!("{}: {:.0}%", self.label, self.value * 100.0);

        let mut ts = TextStyle::new();
        ts.set_color(Color::LIGHT_GRAY);
        ts.set_font_size(18.0);

        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&ts);
        ps.set_text_align(TextAlign::Left);

        let mut builder = ParagraphBuilder::new(&ps, ctx.fonts);
        builder.push_style(&ts);
        builder.add_text(&label_text);

        let mut paragraph = builder.build();
        paragraph.layout(self.rect.width());

        // 画在 Slider 上方 25px 处
        paragraph.paint(canvas, Point::new(self.rect.left, self.rect.top - 25.0));
    }
}