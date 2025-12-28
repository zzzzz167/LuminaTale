use lumina_core::Ctx;
use lumina_ui::{Rect, Color, UiRenderer, Transform};
use crate::core::animator::{RenderSprite, SceneAnimator};

pub struct Painter {
    // Painter 现在不需要持有任何东西
}

impl Painter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn paint(
        &mut self,
        ui: &mut impl UiRenderer,
        _ctx: &Ctx,
        animator: &SceneAnimator,
        window_size: (f32, f32),
    ) {
        let (win_w, win_h) = window_size;

        let mut render_list: Vec<&RenderSprite> = animator.sprites.values().collect();
        render_list.sort_by(|a, b| a.z_index.cmp(&b.z_index));

        for sprite in render_list {
            if sprite.pending_data {
                continue;
            }

            let filename = sprite.full_asset_name();
            let is_bg = sprite.z_index < 0;

            if is_bg {
                let rect = Rect::new(0.0, 0.0, win_w, win_h);
                let alpha = (sprite.alpha * 255.0) as u8;
                ui.draw_image(&filename, rect, Color::rgba(255, 255, 255, alpha));
                continue;
            }

            let (raw_w, raw_h) = ui.measure_image(&filename).unwrap_or((100.0, 100.0));

            let mut t = Transform::default();
            t.x = sprite.pos.x + sprite.offset.x;
            t.y = sprite.pos.y + sprite.offset.y;
            t.rotation = sprite.rotation;
            t.scale_x = sprite.scale;
            t.scale_y = sprite.scale;

            ui.with_transform(t, &mut |ui| {
                let offset_x = -raw_w * sprite.anchor.x;
                let offset_y = -raw_h * sprite.anchor.y;

                let draw_rect = Rect::new(offset_x, offset_y, raw_w, raw_h);

                let alpha = (sprite.alpha * 255.0) as u8;
                ui.draw_image(&filename, draw_rect, Color::rgba(255, 255, 255, alpha));

                //TODO: 这里也许可以实现“点击人物”?
                // if ui.interact(draw_rect).is_clicked() { ... }
            });
        }
    }
}