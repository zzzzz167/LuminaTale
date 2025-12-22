use lumina_core::Ctx;
use skia_safe::textlayout::{FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle};
use skia_safe::{Canvas, Color, Paint, Point, Rect};

use crate::core::assets::AssetManager;
use crate::scene::animator::{RenderSprite, SceneAnimator};

pub struct Painter {
    pub font_collection: FontCollection,
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
        window_size: (f32, f32),
        assets: &mut AssetManager,
    ) {
        canvas.clear(Color::BLACK);

        self.draw_sprites(canvas, animator, assets);

        self.draw_dialogue(canvas, ctx, window_size.0, window_size.1);
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