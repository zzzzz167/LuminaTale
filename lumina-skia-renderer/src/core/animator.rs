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

    pub fn set_prop(&mut self, key: &str, val: f32) {
        match key {
            "x" => self.pos.x = val,
            "y" => self.pos.y = val,
            "alpha" | "opacity" => self.alpha = val.clamp(0.0, 1.0),
            "scale" => self.scale = val,
            "rotation" | "angle" => self.rotation = val,
            _ => {}
        }
    }
}

struct GenericTweener {
    target: String,
    duration: f32,
    elapsed: f32,
    // 存储 (属性名, (起始值, 目标值))
    props: HashMap<String, (f32, f32)>,
    easing: String,
}

pub struct SceneAnimator {
    pub sprites: HashMap<String, RenderSprite>,
    generic_tweens: Vec<GenericTweener>,
    screen_size: (f32, f32),
}

impl SceneAnimator {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            generic_tweens: Vec::new(),
            screen_size: (1920.0, 1080.0),
        }
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.screen_size = (w, h);
    }

    pub fn update(&mut self, dt: f32) {
        let mut finished = Vec::new();

        for (i, tween) in self.generic_tweens.iter_mut().enumerate() {
            tween.elapsed += dt;
            let t = (tween.elapsed / tween.duration).clamp(0.0, 1.0);

            let progress = match tween.easing.as_str() {
                "linear" => t,
                "ease_out" => t * (2.0 - t), // Quad ease out
                "ease_in" => t * t,
                _ => t,
            };

            if let Some(sprite) = self.sprites.get_mut(&tween.target) {
                for (key, (start_val, end_val)) in &tween.props {
                    let current_val = start_val + (end_val - start_val) * progress;
                    sprite.set_prop(key, current_val);
                }
            }

            if t >= 1.0 {
                finished.push(i);
            }
        }

        for i in finished.iter().rev() {
            self.generic_tweens.remove(*i);
        }

        self.sprites.retain(|target, sprite| {
            let is_visible = sprite.alpha > 0.001;
            let has_active_tween = self.generic_tweens.iter().any(|t| t.target == *target);
            is_visible || has_active_tween
        });
    }

    pub fn handle_modify_visual(
        &mut self,
        target: String,
        props: HashMap<String, f32>,
        duration: f32,
        easing: String
    ) {
        if let Some(sprite) = self.sprites.get_mut(&target) {
            if duration <= 0.001 {
                for (k, v) in props {
                    sprite.set_prop(&k, v);
                }
                self.generic_tweens.retain(|t| t.target != target);
            } else {
                let mut tween_props = HashMap::new();
                for (k, target_val) in props {
                    // 获取当前值作为起点
                    let start_val = match k.as_str() {
                        "x" => sprite.pos.x,
                        "y" => sprite.pos.y,
                        "alpha" | "opacity" => sprite.alpha,
                        "scale" => sprite.scale,
                        "rotation" | "angle" => sprite.rotation,
                        _ => continue, // 不支持动画的属性跳过
                    };
                    tween_props.insert(k, (start_val, target_val));
                }
                self.generic_tweens.retain(|t| t.target != target);

                self.generic_tweens.push(GenericTweener {
                    target,
                    duration,
                    elapsed: 0.0,
                    props: tween_props,
                    easing,
                });
            }
        }
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
            let mut props = HashMap::new();
            props.insert("alpha".to_string(), (0.0, 1.0));

            self.generic_tweens.push(GenericTweener {
                target: target.clone(),
                duration: 0.5,
                elapsed: 0.0,
                props,
                easing: "linear".to_string(),
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

                let mut props = HashMap::new();
                props.insert("x".to_string(), (sprite.pos.x, new_x));

                self.generic_tweens.push(GenericTweener {
                    target: target.clone(),
                    duration: 0.5,
                    elapsed: 0.0,
                    props,
                    easing: "ease_out".to_string(),
                });
            }
        }
    }

    pub fn handle_hide_sprite(&mut self, target: String, trans: Option<String>) {
        let should_fade = matches!(trans.as_deref(), Some("fade_out") | Some("dissolve"));

        if should_fade {
            if let Some(sprite) = self.sprites.get(&target) {
                let mut props = HashMap::new();
                props.insert("alpha".to_string(), (sprite.alpha, 0.0));

                self.generic_tweens.push(GenericTweener {
                    target: target.clone(),
                    duration: 0.5,
                    elapsed: 0.0,
                    props,
                    easing: "linear".to_string(),
                });
            }
        } else {
            self.sprites.remove(&target);
            self.generic_tweens.retain(|t| t.target != target);
        }
    }

    pub fn handle_new_scene(&mut self, bg_name: Option<String>, _trans: String) {
        self.sprites.clear();
        self.generic_tweens.clear();

        if let Some(bg) = bg_name {
            // 背景通常没有 attrs，传空 Vec
            let mut bg_sprite = RenderSprite::new("bg".to_string(), bg, vec![]);
            bg_sprite.z_index = -100;
            bg_sprite.anchor = Vec2::new(0.0, 0.0);
            self.sprites.insert("bg".to_string(), bg_sprite);
        }
    }


}
