use std::collections::HashMap;
use lumina_core::event::{LayoutConfig, TransitionConfig};

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
            "scale" | "scale_x" | "scale_y" => self.scale = val, // 确保这里覆盖了所有 Lua 可能发的 key
            "alpha" | "opacity" => self.alpha = val.clamp(0.0, 1.0),
            "rotation" | "angle" => self.rotation = val,
            _ => {
                log::warn!("RenderSprite: Unknown prop '{}'", key);
            }
        }
    }

    pub fn get_prop(&self, key: &str) -> f32 {
        match key {
            "x" => self.pos.x,
            "y" => self.pos.y,
            "alpha" | "opacity" => self.alpha,
            "scale" => self.scale,
            "rotation" | "angle" => self.rotation,
            _ => 0.0,
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

    layouts: HashMap<String, LayoutConfig>,
    trans_registry: HashMap<String, TransitionConfig>,
}

impl SceneAnimator {
    pub fn new() -> Self {
        let mut layouts = HashMap::new();
        layouts.insert("center".into(), LayoutConfig { x: 0.5, y: 1.0, anchor_x: 0.5, anchor_y: 1.0 });
        layouts.insert("left".into(), LayoutConfig { x: 0.2, y: 1.0, anchor_x: 0.5, anchor_y: 1.0 });
        layouts.insert("right".into(), LayoutConfig { x: 0.8, y: 1.0, anchor_x: 0.5, anchor_y: 1.0 });

        Self {
            sprites: HashMap::new(),
            generic_tweens: Vec::new(),
            screen_size: (1920.0, 1080.0),
            layouts,
            trans_registry: HashMap::new(),
        }
    }
    pub fn handle_register_layout(&mut self, name: String, config: LayoutConfig) {
        self.layouts.insert(name, config);
    }
    pub fn handle_register_transition(&mut self, name: String, config: TransitionConfig) {
        self.trans_registry.insert(name, config);
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
                "ease_out" => t * (2.0 - t),
                "ease_in" => t * t,
                "ease_in_out" => if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t },
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
            self.generic_tweens.retain(|t| t.target != target);
            if duration <= 0.001 {
                for (k, v) in props {
                    sprite.set_prop(&k, v);
                }
            } else {
                let mut tween_props = HashMap::new();
                for (k, target_val) in props {
                    tween_props.insert(k.clone(), (sprite.get_prop(&k), target_val));
                }
                self.generic_tweens.push(GenericTweener {
                    target, duration, elapsed: 0.0, props: tween_props, easing
                });
            }
        }
    }

    pub fn handle_new_sprite(&mut self, target: String, texture: String, pos_str: Option<&str>, trans: Option<String>, attrs: Vec<String>) {
        let mut sprite = RenderSprite::new(target.clone(), texture, attrs);

        let layout_key = pos_str.unwrap_or("center");
        let layout = self.layouts.get(layout_key).cloned().unwrap_or(LayoutConfig {
            x: 0.5, y: 1.0, anchor_x: 0.5, anchor_y: 1.0
        });

        let (w, h) = self.screen_size;
        sprite.pos = Vec2::new(layout.x * w, layout.y * h);
        sprite.anchor = Vec2::new(layout.anchor_x, layout.anchor_y);

        if let Some(trans_name) = trans {
            if let Some(cfg) = self.trans_registry.get(&trans_name).cloned() {
                // 有静态配置
                let mut tween_props = HashMap::new();
                for (k, (from_opt, to_val)) in cfg.props {
                    // 如果定义了 from，立即应用初始值
                    if let Some(from_val) = from_opt {
                        sprite.set_prop(&k, from_val);
                        tween_props.insert(k, (from_val, to_val));
                    } else {
                        // 如果没定义 from，取当前值 (对于 NewSprite 来说通常是默认值)
                        tween_props.insert(k.clone(), (sprite.get_prop(&k), to_val));
                    }
                }

                self.generic_tweens.push(GenericTweener {
                    target: target.clone(),
                    duration: cfg.duration,
                    elapsed: 0.0,
                    props: tween_props,
                    easing: cfg.easing,
                });
            }
        }

        self.sprites.insert(target, sprite);
    }

    pub fn handle_update_sprite(&mut self, target: String, trans: String, new_pos: Option<&str>, new_attrs: Vec<String>) {
        if let Some(sprite) = self.sprites.get_mut(&target) {
            // 更新属性
            if !new_attrs.is_empty() {
                sprite.attrs = new_attrs;
            }
            // 计算目标位置 (Layout)
            let target_pos_vec = if let Some(pos_key) = new_pos {
                let layout = self.layouts.get(pos_key).cloned().unwrap_or(LayoutConfig {
                    x: 0.5,
                    y: 1.0,
                    anchor_x: 0.5,
                    anchor_y: 1.0
                });
                let (w, h) = self.screen_size;
                Some(Vec2::new(layout.x * w, layout.y * h))
            } else {
                None
            };

            if !trans.is_empty() {
                if let Some(cfg) = self.trans_registry.get(&trans).cloned() {
                    let mut tween_props = HashMap::new();

                    // 特殊处理：如果 Update 导致位置变化，自动把 "x"/"y" 加入 Tween
                    if let Some(tp) = target_pos_vec {
                        // 除非配置里显式覆盖了 x/y，否则补间过去
                        if !cfg.props.contains_key("x") { tween_props.insert("x".into(), (sprite.pos.x, tp.x)); }
                        if !cfg.props.contains_key("y") { tween_props.insert("y".into(), (sprite.pos.y, tp.y)); }
                    }

                    // 应用配置里的其他属性 (alpha, scale 等)
                    for (k, (from_opt, to_val)) in cfg.props {
                        let start = from_opt.unwrap_or(sprite.get_prop(&k));
                        tween_props.insert(k, (start, to_val));
                    }

                    self.generic_tweens.retain(|t| t.target != target);
                    self.generic_tweens.push(GenericTweener {
                        target: target.clone(),
                        duration: cfg.duration,
                        elapsed: 0.0,
                        props: tween_props,
                        easing: cfg.easing,
                    });
                }
            } else {
                // 无转场 (或 Dynamic 拦截): 瞬移
                if let Some(tp) = target_pos_vec {
                    sprite.pos = tp;
                }
            }
        }
    }

    pub fn handle_hide_sprite(&mut self, target: String, trans: Option<String>) {
        if let Some(t_name) = trans {
            if let Some(cfg) = self.trans_registry.get(&t_name).cloned() {
                let mut tween_props = HashMap::new();
                for (k, (from_opt, to_val)) in cfg.props {
                    let start = from_opt.unwrap_or_else(|| self.sprites.get(&target).map(|s| s.get_prop(&k)).unwrap_or(0.0));
                    tween_props.insert(k, (start, to_val));
                }
                self.generic_tweens.push(GenericTweener {
                    target: target.clone(), duration: cfg.duration, elapsed: 0.0, props: tween_props, easing: cfg.easing
                });
                return;
            }
        }
        self.sprites.remove(&target);
        self.generic_tweens.retain(|t| t.target != target);
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
