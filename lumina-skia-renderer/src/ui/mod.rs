use std::collections::HashMap;
use lumina_ui::input::{Interaction, UiContext};
use lumina_ui::{Alignment, Color, Rect, Style, UiRenderer, Background, Transform, ShaderSpec};
use lumina_ui::types::GradientDirection;
use skia_safe::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextAlign, TextStyle};
use skia_safe::{Canvas, Paint, Point, RRect, Rect as SkRect, gradient_shader::linear, TileMode, RuntimeEffect, Data, SamplingOptions, Matrix, runtime_effect::ChildPtr, shaders};
use crate::core::AssetManager;

pub struct UiDrawer<'a> {
    pub(crate) canvas: &'a Canvas,
    input: &'a UiContext,
    fonts: &'a FontCollection,
    pub assets: &'a mut AssetManager,
    pub time: f32,
    shaders: &'a HashMap<String, RuntimeEffect>,
    transform_stack: Vec<Transform>,
}

impl<'a> UiDrawer<'a> {
    pub fn new(
        canvas: &'a Canvas,
        input: &'a UiContext,
        fonts: &'a FontCollection,
        assets: &'a mut AssetManager,
        time: f32,
        shaders: &'a HashMap<String, RuntimeEffect>,
    ) -> Self {
        Self { canvas, input, fonts, assets, time , transform_stack: Vec::new(),shaders}
    }

    fn to_skia_rect(&self, r: Rect) -> SkRect {
        SkRect::new(r.x, r.y, r.x + r.w, r.y + r.h)
    }

    fn to_skia_color(&self, c: Color) -> skia_safe::Color {
        skia_safe::Color::from_argb(c.a, c.r, c.g, c.b)
    }

    fn get_local_mouse_pos(&self) -> (f32, f32) {
        let (mut mx, mut my) = self.input.mouse_pos;

        for t in &self.transform_stack {
            // 1. 逆平移
            mx -= t.x;
            my -= t.y;

            // 2. 逆旋转
            if t.rotation != 0.0 {
                let rad = -t.rotation.to_radians(); // 反向旋转
                let cos = rad.cos();
                let sin = rad.sin();
                let nx = mx * cos - my * sin;
                let ny = mx * sin + my * cos;
                mx = nx;
                my = ny;
            }

            // 3. 逆缩放
            if t.scale_x != 0.0 { mx /= t.scale_x; }
            if t.scale_y != 0.0 { my /= t.scale_y; }
        }

        (mx, my)
    }
}

impl <'a> UiRenderer for UiDrawer<'a> {
    fn draw_style(&mut self, rect: Rect, style: &Style) {
        let sk_rect = self.to_skia_rect(rect);
        let radius = style.border.radius;
        let rrect = RRect::new_rect_xy(sk_rect, radius, radius);

        match &style.background {
            Background::Solid(c) => {
                let mut paint = Paint::default();
                paint.set_color(self.to_skia_color(*c));
                paint.set_anti_alias(true);
                self.canvas.draw_rrect(rrect, &paint);
            }
            Background::LinearGradient {dir, colors} => {
                let (start_color, end_color) = colors;
                let sk_colors = [
                    self.to_skia_color(*start_color),
                    self.to_skia_color(*end_color),
                ];

                let (start_pt, end_pt) = match dir {
                    GradientDirection::Vertical => (
                        Point::new(sk_rect.center_x(), sk_rect.top()),
                        Point::new(sk_rect.center_x(), sk_rect.bottom()),
                    ),
                    GradientDirection::Horizontal => (
                        Point::new(sk_rect.left(), sk_rect.center_y()),
                        Point::new(sk_rect.right(), sk_rect.center_y()),
                    ),
                    GradientDirection::Diagonal => (
                        Point::new(sk_rect.left(), sk_rect.top()),
                        Point::new(sk_rect.right(), sk_rect.bottom()),
                    ),
                    GradientDirection::InverseDiagonal => (
                        Point::new(sk_rect.right(), sk_rect.top()),
                        Point::new(sk_rect.left(), sk_rect.bottom()),
                    ),
                };

                let shader = linear(
                    (start_pt, end_pt),
                    sk_colors.as_slice(),
                    None,
                    TileMode::Clamp,
                    None,
                    None,
                );

                let mut paint = Paint::default();
                paint.set_shader(shader);
                paint.set_anti_alias(true);
                self.canvas.draw_rrect(rrect, &paint);
            }
            Background::Image(id) => {
                self.canvas.save();
                self.canvas.clip_rrect(rrect, None, true);

                self.draw_image(id, rect, Color::WHITE);

                self.canvas.restore();
            }
            Background::None => {}
        }

        if style.border.width > 0.0 && style.border.color.a > 0 {
            let mut paint = Paint::default();
            paint.set_style(skia_safe::paint::Style::Stroke);
            paint.set_stroke_width(style.border.width);
            paint.set_color(self.to_skia_color(style.border.color));
            paint.set_anti_alias(true);

            self.canvas.draw_rrect(rrect, &paint);
        }
    }

    fn draw_image(&mut self, image_id: &str, rect: Rect, tint: Color) {
        if let Some(sk_image) = self.assets.get_image(image_id) {
            let sk_rect = self.to_skia_rect(rect);
            let mut paint = Paint::default();
            paint.set_color(self.to_skia_color(tint));

            paint.set_anti_alias(true);

            self.canvas.draw_image_rect(sk_image, None, sk_rect, &paint);
        }
    }

    fn draw_text(&mut self, text: &str, rect: Rect, color: Color, size: f32, align: Alignment, font: Option<&str>) {
        let mut ts = TextStyle::new();
        ts.set_color(self.to_skia_color(color));
        ts.set_font_size(size);
        if let Some(font_name) = font {
            ts.set_font_families(&[font_name]);
        }

        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&ts);

        let skia_align = match align {
            Alignment::Start => TextAlign::Left,
            Alignment::Center => TextAlign::Center,
            Alignment::End => TextAlign::Right,
        };
        ps.set_text_align(skia_align);

        let mut builder = ParagraphBuilder::new(&ps, self.fonts);
        builder.push_style(&ts);
        builder.add_text(text);

        let mut paragraph = builder.build();
        paragraph.layout(rect.w);

        // 垂直居中计算
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
        let (mx, my) = self.get_local_mouse_pos();
        let hovered = rect.contains(mx, my);

        if hovered {
            if self.input.mouse_pressed {
                return Interaction::Clicked;
            }
            if self.input.mouse_held {
                return Interaction::Held;
            }
            return Interaction::Hovered;
        }

        Interaction::None
    }

    fn cursor_pos(&self) -> (f32, f32) {
        self.input.mouse_pos
    }

    fn with_transform(&mut self, t: Transform, f: &mut dyn FnMut(&mut Self)) {
        self.canvas.save();
        self.canvas.translate((t.x, t.y));
        self.canvas.rotate(t.rotation, None);
        self.canvas.scale((t.scale_x, t.scale_y));
        self.transform_stack.push(t);
        f(self);
        self.transform_stack.pop();
        self.canvas.restore();
    }

    fn time(&self) -> f32 {
        self.time
    }

    fn measure_image(&mut self, image_id: &str) -> Option<(f32, f32)> {
        self.assets.get_image(image_id).map(|img| (img.width() as f32, img.height() as f32))
    }

    fn draw_shader(&mut self, rect: Rect, spec: ShaderSpec) {
        let effect = match self.shaders.get(spec.shader_id) {
            Some(e) => e,
            None => {
                log::warn!("Shader not found: {}", spec.shader_id);
                return;
            }
        };
        let mut uniform_bytes = Vec::new();
        for &val in spec.uniforms {
            uniform_bytes.extend_from_slice(&val.to_le_bytes());
        }
        let data = Data::new_copy(&uniform_bytes);

        let mut children: Vec<ChildPtr> = Vec::new();
        for &img_id in spec.images {
            // 从 AssetManager 获取 Image
            let fallback_shader = shaders::color(skia_safe::Color::TRANSPARENT);
            let shader = if let Some(image) = self.assets.get_image(img_id) {
                let img_w = image.width() as f32;
                let img_h = image.height() as f32;

                // 防止除以 0
                if img_w > 0.0 && img_h > 0.0 {
                    let scale_x = rect.w / img_w;
                    let scale_y = rect.h / img_h;

                    // 构建矩阵：先缩放，再平移
                    let mut matrix = Matrix::new_identity();
                    matrix.set_scale_translate((scale_x, scale_y), (rect.x, rect.y));

                    // 使用带矩阵的 Shader
                    image.to_shader(None, SamplingOptions::default(), &matrix)
                        .unwrap_or(fallback_shader)
                } else {
                    fallback_shader
                }
            } else {
                fallback_shader
            };
            children.push(shader.into());
        }
        if let Some(shader) = effect.make_shader(data, &children, None) {
            let mut paint = Paint::default();
            paint.set_shader(Some(shader));
            paint.set_anti_alias(true);

            // 转换坐标
            let sk_rect = self.to_skia_rect(rect);
            self.canvas.draw_rect(sk_rect, &paint);
        }
    }
}