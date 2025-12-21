use skia_safe::{Rect, Paint, Point, Color};
use crate::ui::{RenderContext, WidgetRender};

#[derive(Debug, Clone, Copy)]
pub enum BoxFit {
    Contain,
    Cover,
    Fill,
}

#[derive(Clone)]
pub struct Image {
    pub path: String,
    pub fit: BoxFit,
    pub rect: Rect,
}

impl Image {
    pub fn new(path: &str, fit: BoxFit) -> Self {
        Self {
            path: path.to_string(),
            fit,
            rect: Rect::default(),
        }
    }
}

impl WidgetRender for Image {
    fn render(&self, ctx: &mut RenderContext) {
        if let Some(sk_image) = ctx.assets.get_image(&self.path) {
            let src_rect = Rect::from_wh(sk_image.width() as f32, sk_image.height() as f32);
            let dst_rect = self.rect;

            let mut paint = Paint::default();
            paint.set_anti_alias(true);

            // TODO: 这里可以实现更复杂的 BoxFit 逻辑 (计算 src_rect 子集)
            ctx.canvas.draw_image_rect(
                sk_image,
                Some((&src_rect, skia_safe::canvas::SrcRectConstraint::Strict)),
                dst_rect,
                &paint
            );
        } else {
            let mut paint = Paint::default();
            paint.set_color(Color::RED);
            paint.set_style(skia_safe::paint::Style::Stroke);
            paint.set_stroke_width(2.0);

            ctx.canvas.draw_rect(self.rect, &paint);
            ctx.canvas.draw_line(
                Point::new(self.rect.left, self.rect.top),
                Point::new(self.rect.right, self.rect.bottom),
                &paint
            );
            ctx.canvas.draw_line(
                Point::new(self.rect.right, self.rect.top),
                Point::new(self.rect.left, self.rect.bottom),
                &paint
            );
        }
    }
}