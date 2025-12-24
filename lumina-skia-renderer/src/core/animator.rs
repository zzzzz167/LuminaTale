use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
}

#[derive(Clone, Debug)]
pub struct RenderSprite {
    pub target: String,
    pub texture: String,
    pub attrs: Vec<String>,

    pub pos: Vec2,
    pub scale: f32,
    pub alpha: f32,
    pub rotation: f32,
    pub anchor: Vec2,
    pub z_index: i32,
}

impl RenderSprite {
    pub fn new(target: String, texture: String, attrs: Vec<String>) -> Self {
        Self {
            target,
            texture,
            attrs,
            pos: Vec2::new(0.0, 0.0),
            scale: 1.0,
            alpha: 1.0,
            rotation: 0.0,
            anchor: Vec2::new(0.5, 1.0),
            z_index: 0,
        }
    }
    pub fn full_asset_name(&self) -> String {
        if self.attrs.is_empty() {
            return self.texture.clone();
        }
        let mut name = self.texture.clone();
        for attr in &self.attrs {
            name.push('_');
            name.push_str(attr);
        }
        name
    }
}

struct Tweener {
    target: String,
    duration: f32,
    elapsed: f32,
    start_pos: Vec2,
    end_pos: Vec2,
    start_alpha: f32,
    end_alpha: f32,
}

pub struct SceneAnimator {
    pub sprites: HashMap<String, RenderSprite>,
    tweens: Vec<Tweener>,
    screen_size: (f32, f32),
}

impl SceneAnimator {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            tweens: Vec::new(),
            screen_size: (1920.0, 1080.0),
        }
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.screen_size = (w, h);
    }

    pub fn update(&mut self, dt: f32) {
        let mut finished = Vec::new();
        for (i, tween) in self.tweens.iter_mut().enumerate() {
            tween.elapsed += dt;
            let t = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
            let ease = t * (2.0 - t);

            if let Some(sprite) = self.sprites.get_mut(&tween.target) {
                sprite.pos.x = tween.start_pos.x + (tween.end_pos.x - tween.start_pos.x) * ease;
                sprite.pos.y = tween.start_pos.y + (tween.end_pos.y - tween.start_pos.y) * ease;
                sprite.alpha = tween.start_alpha + (tween.end_alpha - tween.start_alpha) * ease;
            }

            if t >= 1.0 {
                finished.push(i);
            }
        }

        for i in finished.iter().rev() {
            self.tweens.remove(*i);
        }

        self.sprites.retain(|target, sprite| {
            let is_visible = sprite.alpha > 0.001;
            let has_active_tween = self.tweens.iter().any(|t| t.target == *target);
            is_visible || has_active_tween
        });
    }

    pub fn handle_new_sprite(&mut self, target: String, texture: String, trans: String, pos_str: Option<&str>, attrs: Vec<String>) {
        let mut sprite = RenderSprite::new(target.clone(), texture, attrs);

        let (w, h) = self.screen_size;
        let x = match pos_str.unwrap_or("center") {
            "left" => w * 0.2,
            "right" => w * 0.8,
            _ => w * 0.5,
        };
        sprite.pos = Vec2::new(x, h);

        if trans == "fade_in" || trans == "dissolve" {
            sprite.alpha = 0.0;
            self.tweens.push(Tweener {
                target: target.clone(),
                duration: 0.5,
                elapsed: 0.0,
                start_pos: sprite.pos,
                end_pos: sprite.pos,
                start_alpha: 0.0,
                end_alpha: 1.0,
            });
        }

        self.sprites.insert(target, sprite);
    }

    pub fn handle_update_sprite(&mut self, target: String, _trans: String, pos_str: Option<&str>, attrs: Vec<String>) {
        if let Some(sprite) = self.sprites.get_mut(&target) {
            if !attrs.is_empty() {
                sprite.attrs = attrs;
            }

            // 处理位置变更动画
            if let Some(pos) = pos_str {
                let (w, _) = self.screen_size;
                let new_x = match pos {
                    "left" => w * 0.2,
                    "right" => w * 0.8,
                    _ => w * 0.5,
                };

                self.tweens.push(Tweener {
                    target: target.clone(),
                    duration: 0.5,
                    elapsed: 0.0,
                    start_pos: sprite.pos,
                    end_pos: Vec2::new(new_x, sprite.pos.y),
                    start_alpha: sprite.alpha,
                    end_alpha: sprite.alpha,
                });
            }
        }
    }

    pub fn handle_hide_sprite(&mut self, target: String, trans: Option<String>) {
        let should_fade = matches!(trans.as_deref(), Some("fade_out") | Some("dissolve"));

        if should_fade {
            if let Some(sprite) = self.sprites.get(&target) {
                self.tweens.push(Tweener {
                    target: target.clone(),
                    duration: 0.5,
                    elapsed: 0.0,
                    start_pos: sprite.pos,
                    end_pos: sprite.pos,
                    start_alpha: sprite.alpha,
                    end_alpha: 0.0,
                });
            }
        } else {
            self.sprites.remove(&target);
            self.tweens.retain(|t| t.target != target);
        }
    }

    pub fn handle_new_scene(&mut self, bg_name: Option<String>, _trans: String) {
        self.sprites.clear();
        self.tweens.clear();

        if let Some(bg) = bg_name {
            // 背景通常没有 attrs，传空 Vec
            let mut bg_sprite = RenderSprite::new("bg".to_string(), bg, vec![]);
            bg_sprite.z_index = -100;
            bg_sprite.anchor = Vec2::new(0.0, 0.0);
            self.sprites.insert("bg".to_string(), bg_sprite);
        }
    }


}
