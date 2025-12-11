use skia_safe::{Canvas, Color, FontStyle, Paint, Point, Rect};
use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle};
use lumina_core::Ctx;
use crate::assets::AssetManager;

pub struct Painter {
    assets: AssetManager,
    font_collection: FontCollection,
}

impl Painter {
    pub fn new() -> Self{
        let assets = AssetManager::new("example-game/assets");
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(skia_safe::FontMgr::default(), None);

        Self {
            assets,
            font_collection,
        }
    }

    pub fn paint(&mut self, canvas: &Canvas, ctx: &Ctx, window_size: (f32, f32)) {
        let (w, h) = window_size;

        canvas.clear(Color::WHITE);

        for layer_name in &ctx.layer_record.arrange {
            if let Some(sprites) = ctx.layer_record.layer.get(layer_name) {
                for sprite in sprites {
                    // --- 自动拼接文件名 ---
                    let mut filename = sprite.target.clone();
                    if !sprite.attrs.is_empty() {
                        filename.push('_');
                        filename.push_str(&sprite.attrs.join("_"));
                    }

                    if let Some(image) = self.assets.get_image(&filename) {
                        let img_w = image.width() as f32;
                        let img_h = image.height() as f32;

                        if img_w == 0.0 || img_h == 0.0 { continue; }

                        let dest_rect = if sprite.zindex == 0 {
                            let scale = (w / img_w).max(h / img_h);
                            let draw_w = img_w * scale;
                            let draw_h = img_h * scale;

                            let x = (w - draw_w) / 2.0;
                            let y = (h - draw_h) / 2.0;

                            Rect::new(x, y, x + draw_w, y + draw_h)
                        } else {
                            let target_height = h * 0.85;
                            let scale = target_height / img_h;
                            let draw_w = img_w * scale;
                            let draw_h = img_h * scale;

                            let pos_str = sprite.position.as_deref().unwrap_or("center");

                            let center_x = match pos_str {
                                "left" => w * 0.2,   // 放在屏幕左侧 20% 处
                                "right" => w * 0.8,  // 放在屏幕右侧 80% 处
                                "center" | _ => w * 0.5, // 屏幕正中
                            };

                            // 计算左上角坐标 (以底部对齐)
                            let x = center_x - (draw_w / 2.0);
                            let y = h - draw_h; // 紧贴底部

                            Rect::new(x, y, x + draw_w, y + draw_h)
                        };

                        canvas.draw_image_rect(
                            &image,
                            None,
                            dest_rect,
                            &Paint::default()
                        );
                    }
                }
            }
        }

        self.draw_ui(canvas, ctx, w, h);
    }

    fn draw_ui(&mut self, canvas: &Canvas, ctx: &Ctx, w: f32, h: f32) {
        if let Some(last_dialogue) = ctx.dialogue_history.last() {
            let ui_height = h * 0.3;
            let ui_rect = Rect::new(0.0, h - ui_height, w, h);
            let mut bg_paint = Paint::default();
            bg_paint.set_color(Color::from_argb(200, 0, 0, 0));
            canvas.draw_rect(ui_rect, &bg_paint);

            let mut text_style = TextStyle::new();
            text_style.set_color(Color::WHITE);
            text_style.set_font_size(24.0);
            text_style.set_font_style(FontStyle::new(Weight::NORMAL, Width::NORMAL, Slant::Upright));

            let mut para_style = ParagraphStyle::new();
            para_style.set_text_style(&text_style);

            if let Some(name) = &last_dialogue.speaker {
                let mut name_style = text_style.clone();
                name_style.set_color(Color::YELLOW);
                name_style.set_font_size(30.0);

                let mut builder = ParagraphBuilder::new(&para_style, &self.font_collection);
                builder.push_style(&name_style);
                builder.add_text(name);
                let mut paragraph = builder.build();
                paragraph.layout(w);
                paragraph.paint(canvas, Point::new(20.0, h - ui_height + 20.0));
            }

            let mut builder = ParagraphBuilder::new(&para_style, &self.font_collection);
            builder.push_style(&text_style);
            builder.add_text(&last_dialogue.text);
            let mut paragraph = builder.build();
            paragraph.layout(w - 80.0);
            paragraph.paint(canvas, Point::new(40.0, h - ui_height + 70.0));
        }
    }
}