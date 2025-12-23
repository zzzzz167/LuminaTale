use lumina_core::Ctx;
use skia_safe::{Canvas, Paint, Rect};
use crate::core::{animator::{RenderSprite, SceneAnimator}, AssetManager};

pub struct Painter {}

impl Painter {
    pub fn new() -> Self{
        Self {}
    }

    pub fn paint(
        &mut self,
        canvas: &Canvas,
        _ctx: &Ctx,
        animator: &SceneAnimator,
        _window_size: (f32, f32),
        assets: &mut AssetManager,
    ) {
        //canvas.clear(Color::BLACK);

        self.draw_sprites(canvas, animator, assets);
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
}