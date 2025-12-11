use skia_safe::{Canvas, Color, FontStyle, Paint, Point, Rect};
use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextAlign, TextStyle};
use lumina_core::Ctx;
use crate::assets::AssetManager;
use crate::ui_state::{UiState, UiMode};

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

    pub fn paint(&mut self, canvas: &Canvas, ctx: &Ctx, ui_state: &mut UiState, window_size: (f32, f32)) {
        let (w, h) = window_size;

        canvas.clear(Color::WHITE);

        self.draw_layers(canvas, ctx, w, h);

        match &mut ui_state.mode {
            UiMode::Choice {title, options, hit_boxes, hover_index} => {
                let mut mask = Paint::default();
                mask.set_color(Color::from_argb(128, 0, 0, 0));
                canvas.draw_rect(Rect::new(0.0, 0.0, w, h), &mask);

                hit_boxes.clear();

                self.draw_choice_menu(
                    canvas,
                    w, h,
                    title.as_deref(),
                    options,
                    hit_boxes,
                    *hover_index,
                );
            },

            _ => {
                self.draw_dialogue(canvas, ctx, w, h);
            }
        }

    }

    fn draw_layers(&mut self, canvas: &Canvas, ctx: &Ctx, w: f32, h: f32) {
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
    }

    fn draw_choice_menu(
        &mut self,
        canvas: &Canvas,
        w: f32, h: f32,
        title: Option<&str>,
        options: &[String],
        hit_boxes: &mut Vec<Rect>,
        hover_index: Option<usize>
    ) {
        let btn_width = 500.0;
        let btn_height = 70.0;
        let gap = 20.0;

        let total_h = options.len() as f32 * (btn_height + gap) - gap;
        let start_y = (h - total_h) / 2.0;
        let center_x = w / 2.0;

        if let Some(t) = title {
            // 简单绘制在菜单上方
            let mut title_style = TextStyle::new();
            title_style.set_color(Color::WHITE);
            title_style.set_font_size(40.0);
            title_style.set_font_style(FontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright));

            let mut pb = ParagraphBuilder::new(&ParagraphStyle::new(), &self.font_collection);
            pb.push_style(&title_style);
            pb.add_text(t);
            let mut p = pb.build();
            p.layout(w);
            p.paint(canvas, Point::new((w - p.max_width()) / 2.0, start_y - 80.0));
        }

        for (i, opt_text) in options.iter().enumerate() {
            let y = start_y + i as f32 * (btn_height + gap);
            let rect = Rect::from_xywh(center_x - btn_width / 2.0, y, btn_width, btn_height);

            // [关键] 将计算出的区域存入 hit_boxes
            hit_boxes.push(rect);

            let is_hover = hover_index == Some(i);

            // 按钮背景
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            if is_hover {
                paint.set_color(Color::from_rgb(100, 149, 237)); // 悬停：矢车菊蓝
            } else {
                paint.set_color(Color::from_argb(220, 50, 50, 50)); // 普通：深灰
            }
            canvas.draw_round_rect(rect, 12.0, 12.0, &paint);

            // 按钮描边
            let mut stroke = Paint::default();
            stroke.set_style(skia_safe::paint::Style::Stroke);
            stroke.set_stroke_width(2.0);
            stroke.set_color(if is_hover { Color::WHITE } else { Color::GRAY });
            stroke.set_anti_alias(true);
            canvas.draw_round_rect(rect, 12.0, 12.0, &stroke);

            // 按钮文字 (居中)
            let mut ts = TextStyle::new();
            ts.set_color(Color::WHITE);
            ts.set_font_size(28.0);

            let mut ps = ParagraphStyle::new();
            ps.set_text_align(TextAlign::Center); // 文本内部居中
            ps.set_text_style(&ts);

            let mut builder = ParagraphBuilder::new(&ps, &self.font_collection);
            builder.push_style(&ts);
            builder.add_text(opt_text);
            let mut paragraph = builder.build();

            paragraph.layout(btn_width);

            // 计算文字垂直居中
            let text_y = rect.y() + (btn_height - paragraph.height()) / 2.0;
            paragraph.paint(canvas, Point::new(rect.x(), text_y));
        }
    }

    fn draw_dialogue(&mut self, canvas: &Canvas, ctx: &Ctx, w: f32, h: f32) {
        if let Some(last_dialogue) = ctx.dialogue_history.last() {
            let ui_height = h * 0.3;
            let ui_rect = Rect::new(0.0, h - ui_height, w, h);

            let mut bg_paint = Paint::default();
            bg_paint.set_color(Color::from_argb(200, 0, 0, 0));
            canvas.draw_rect(ui_rect, &bg_paint);

            let mut ts = TextStyle::new();
            ts.set_color(Color::WHITE);
            ts.set_font_size(24.0);
            let mut ps = ParagraphStyle::new();
            ps.set_text_style(&ts);

            // Speaker Name
            if let Some(name) = &last_dialogue.speaker {
                let mut name_ts = ts.clone();
                name_ts.set_color(Color::YELLOW);
                name_ts.set_font_size(30.0);

                let mut pb = ParagraphBuilder::new(&ps, &self.font_collection);
                pb.push_style(&name_ts);
                pb.add_text(name);
                let mut p = pb.build();
                p.layout(w);
                p.paint(canvas, Point::new(40.0, h - ui_height + 30.0));
            }

            // Dialogue Text
            let mut pb = ParagraphBuilder::new(&ps, &self.font_collection);
            pb.push_style(&ts);
            pb.add_text(&last_dialogue.text);
            let mut p = pb.build();
            p.layout(w - 100.0);
            p.paint(canvas, Point::new(60.0, h - ui_height + 80.0));
        }
    }
}