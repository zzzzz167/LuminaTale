use skia_safe::{Canvas, Color, FontStyle, Paint, Point, Rect};
use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextAlign, TextStyle};
use lumina_core::Ctx;
use crate::core::assets::AssetManager;
use crate::ui::{UiState, UiMode};
use crate::scene::animator::{RenderSprite, SceneAnimator};
use crate::ui::widgets::{Button, ButtonState};

pub struct Painter {
    font_collection: FontCollection,
}

impl Painter {
    pub fn new() -> Self{
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(skia_safe::FontMgr::default(), None);

        Self {
            font_collection,
        }
    }

    pub fn paint(
        &mut self,
        canvas: &Canvas,
        ctx: &Ctx,
        animator: &SceneAnimator,
        ui_state: &mut UiState,
        window_size: (f32, f32),
        assets: &mut AssetManager
    ) {
        let (w, h) = window_size;

        canvas.clear(Color::BLACK);

        self.draw_sprites(canvas, animator, assets);

        match &mut ui_state.mode {
            UiMode::Choice {title, buttons} => {
                let mut mask = Paint::default();
                mask.set_color(Color::from_argb(128, 0, 0, 0));
                canvas.draw_rect(Rect::new(0.0, 0.0, w, h), &mask);

                if let Some(t) = title {
                    let btn_width = 500.0;
                    let btn_height = 70.0;
                    let gap = 20.0;

                    let total_h = buttons.len() as f32 * (btn_height + gap) - gap;
                    let start_y = (h - total_h) / 2.0;

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

                for btn in buttons {
                    self.draw_button(canvas, btn);
                }
            },
            _ => {
                self.draw_dialogue(canvas, ctx, w, h);
            }
        }
    }

    pub(crate) fn draw_button(&mut self, canvas: &Canvas, btn: &Button) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        let bg_color = match btn.state {
            ButtonState::Normal => Color::from_rgb(60, 60, 60),
            ButtonState::Hovered => Color::from_rgb(80, 80, 100),
            ButtonState::Pressed => Color::from_rgb(40, 40, 40),
        };
        paint.set_color(bg_color);

        canvas.draw_round_rect(btn.rect, 10.0, 10.0, &paint);

        if btn.state == ButtonState::Hovered {
            let mut stroke = Paint::default();
            stroke.set_style(skia_safe::paint::Style::Stroke);
            stroke.set_stroke_width(2.0);
            stroke.set_color(Color::WHITE);
            stroke.set_anti_alias(true);
            canvas.draw_round_rect(btn.rect, 10.0, 10.0, &stroke);
        }

        let mut ts = TextStyle::new();
        ts.set_color(Color::WHITE);
        ts.set_font_size(24.0);
        let mut ps = ParagraphStyle::new();
        ps.set_text_align(TextAlign::Center);
        ps.set_text_style(&ts);

        let mut builder = ParagraphBuilder::new(&ps, &self.font_collection);
        builder.push_style(&ts);
        builder.add_text(&btn.text);

        let mut paragraph = builder.build();
        paragraph.layout(btn.rect.width());

        let text_h = paragraph.height();
        let text_y = btn.rect.y() + (btn.rect.height() - text_h) / 2.0;

        paragraph.paint(canvas, Point::new(btn.rect.x(), text_y));
    }

    fn draw_sprites(&mut self, canvas: &Canvas, animator: &SceneAnimator, assets: &mut AssetManager) {
        let logical_size = animator.window_logical_size;
        let mut render_list: Vec<&RenderSprite> = animator.sprites.values().collect();
        render_list.sort_by(|a, b| a.z_index.cmp(&b.z_index));
        for sprite in render_list {
            let is_bg = sprite.z_index == 0;
            self.draw_single_sprite(canvas, sprite, is_bg, logical_size, assets);
        }
    }

    fn draw_single_sprite(
        &mut self,
        canvas: &Canvas,
        sprite: &RenderSprite,
        is_bg: bool,
        logical_size: (f32, f32),
        assets: &mut AssetManager
    ) {
        let filename = sprite.full_asset_name();

        if let Some(image) = assets.get_image(&filename) {
            let mut paint = Paint::default();
            paint.set_alpha_f(sprite.alpha);

            let dest_rect = if is_bg {
                Rect::from_wh(logical_size.0, logical_size.1)
            } else {
                let img_w = image.width() as f32;
                let img_h = image.height() as f32;

                let draw_w = img_w * sprite.scale;
                let draw_h = img_h * sprite.scale;

                let x = sprite.pos.x - (draw_w * sprite.anchor.x);
                let y = sprite.pos.y - (draw_h * sprite.anchor.y);

                Rect::new(x, y, x + draw_w, y + draw_h)
            };

            canvas.draw_image_rect(&image, None, dest_rect, &paint);
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