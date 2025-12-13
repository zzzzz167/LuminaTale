use std::collections::HashMap;
use std::time::SystemTime;
use skia_safe::Point;

#[derive(Debug, Clone)]
pub struct Tween {
    start: f32,
    end: f32,
    duration: f32,
    elapsed: f32,
}

impl Tween {
    pub fn new(start: f32, end: f32, duration: f32) -> Self {
        Self { start, end, duration, elapsed: 0.0 }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        self.elapsed >= self.duration
    }

    pub fn value(&self) -> f32 {
        if self.duration <= 0.0 { return self.end; }
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        self.start + (self.end - self.start) * t
    }
}

#[derive(Debug, Clone)]
pub struct PosTween {
    start: Point,
    end: Point,
    duration: f32,
    elapsed: f32,
}

impl PosTween {
    pub fn new(start: Point, end: Point, duration: f32) -> Self {
        Self { start, end, duration, elapsed: 0.0 }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        self.elapsed >= self.duration
    }

    pub fn value(&self) -> Point {
        if self.duration <= 0.0 { return self.end; }
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);

        let t = 1.0 - (1.0 - t) * (1.0 - t);

        Point::new(
            self.start.x + (self.end.x - self.start.x) * t,
            self.start.y + (self.end.y - self.start.y) * t,
        )
    }
}

#[derive(Debug, Clone)]
pub struct RenderSprite {
    pub texture_name: String,
    pub attrs: Vec<String>,
    pub z_index: usize,

    pub alpha: f32,
    pub pos: Point,
    pub scale: f32,

    pub alpha_tween: Option<Tween>,
    pub pos_tween: Option<PosTween>,

    pub pending_kill: bool,
}

impl RenderSprite {
    pub fn new(texture_name: String, attrs: Vec<String>, pos: Point, z_index: usize) -> Self {
        Self {
            texture_name,
            attrs,
            z_index,
            alpha: 1.0,
            pos,
            scale: 1.0,
            alpha_tween: None,
            pos_tween: None,
            pending_kill: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(tween) = &mut self.alpha_tween {
            let finished = tween.update(dt);
            self.alpha = tween.value();
            if finished {
                self.alpha_tween = None;
            }
        }

        if let Some(tween) = &mut self.pos_tween {
            let finished = tween.update(dt);
            self.pos = tween.value();
            if finished { self.pos_tween = None; }
        }
    }

    pub fn full_asset_name(&self) -> String {
        if self.attrs.is_empty() {
            self.texture_name.clone()
        } else {
            format!("{}_{}", self.texture_name, self.attrs.join("_"))
        }
    }

    pub fn set_attrs(&mut self, new_attrs: Vec<String>) {
        if self.attrs != new_attrs { self.attrs = new_attrs; }
    }

    pub fn move_to(&mut self, target_pos: Point, duration: f32) {
        let dist = ((self.pos.x - target_pos.x).powi(2) + (self.pos.y - target_pos.y).powi(2)).sqrt();
        if dist < 1.0 {
            self.pos = target_pos;
            self.pos_tween = None;
            return;
        }

        self.pos_tween = Some(PosTween::new(self.pos, target_pos, duration));
    }
}

pub struct SceneAnimator {
    pub sprites: HashMap<String, RenderSprite>,
    pub window_logical_size: (f32, f32),
}

impl SceneAnimator {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            window_logical_size: (1280.0, 720.0),
        }
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.window_logical_size = (w, h);
    }

    pub fn update(&mut self, dt: f32) {
        for sprite in self.sprites.values_mut() {
            sprite.update(dt);
        }

        self.sprites.retain(|_, sprite| {
            // 如果标记了死亡，且所有动画都播放完毕，则移除
            !(sprite.pending_kill && sprite.alpha_tween.is_none() && sprite.pos_tween.is_none())
        });
    }

    pub fn handle_new_sprite(&mut self, id: String, texture_name: String, _transition: String, pos_str: Option<&str>) {
        let (w, h) = self.window_logical_size;

        let pos_x = match pos_str.unwrap_or("center") {
            "left" => w * 0.2,
            "right" => w * 0.8,
            "center" | _ => w * 0.5,
        };
        let pos_y = h;

        let mut sprite = RenderSprite::new(texture_name, vec![], Point::new(pos_x, pos_y), 10);

        if _transition == "dissolve" || _transition == "fade_in" {
            sprite.alpha = 0.0;
            sprite.alpha_tween = Some(Tween::new(0.0, 1.0, 0.5));
        }

        self.sprites.insert(id, sprite);
    }

    pub fn handle_update_sprite(&mut self, id: String, _transition: String, pos_str: Option<&str>, new_attrs: Option<Vec<String>>) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            // 1. 如果有位置变更请求
            if let Some(p_str) = pos_str {
                let (w, h) = self.window_logical_size;
                let target_x = match p_str {
                    "left" => w * 0.2,
                    "right" => w * 0.8,
                    "center" | _ => w * 0.5,
                };
                let target_y = h;

                let target_pos = Point::new(target_x, target_y);
                sprite.move_to(target_pos, 0.5);
            }
            if let Some(attrs) = new_attrs {
                sprite.set_attrs(attrs);
            }
        }
    }

    pub fn handle_hide_sprite(&mut self, id: String, transition: Option<String>) {
        if let Some(sprite) = self.sprites.get_mut(&id) {
            sprite.pending_kill = true;
            let trans = transition.as_deref().unwrap_or("default");
            if trans == "dissolve" || trans == "fade_out" {
                sprite.alpha_tween = Some(Tween::new(sprite.alpha, 0.0, 0.5));
            } else {
                sprite.alpha = 0.0;
            }
        }
    }

    pub fn handle_new_scene(&mut self, texture_name: Option<String>, transition: String) {
        let (w, h) = self.window_logical_size;
        let bg_key = "__scene_bg__";

        let kill_trans = if transition == "dissolve" { "dissolve" } else { "default" };

        for (_key, sprite) in self.sprites.iter_mut() {
            sprite.pending_kill = true;
            if kill_trans == "dissolve" {
                sprite.alpha_tween = Some(Tween::new(sprite.alpha, 0.0, 0.5));
            } else {
                sprite.alpha = 0.0;
            }
        }

        if let Some(old_bg) = self.sprites.remove(bg_key) {
            let dead_key = format!("__dead_bg_{:?}", SystemTime::now());
            self.sprites.insert(dead_key, old_bg);
        }

        if let Some(tex) = texture_name {
            let mut bg = RenderSprite::new(tex, vec![], Point::new(w/2.0, h/2.0), 0);

            if transition == "dissolve" {
                bg.alpha = 0.0;
                bg.alpha_tween = Some(Tween::new(0.0, 1.0, 0.5));
            }
            self.sprites.insert(bg_key.to_string(), bg);
        }
    }
}