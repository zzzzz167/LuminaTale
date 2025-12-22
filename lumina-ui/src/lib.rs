pub mod input;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w &&
            y >= self.y && y < self.y + self.h
    }

    pub fn split_top(&self, height: f32) -> (Rect, Rect) {
        let h = height.min(self.h);
        let top = Rect::new(self.x, self.y, self.w, h);
        let rest = Rect::new(self.x, self.y + h, self.w, self.h - h);
        (top, rest)
    }

    pub fn split_bottom(&self, height: f32) -> (Rect, Rect) {
        let h = height.min(self.h);
        let rest_h = self.h - h;
        let rest = Rect::new(self.x, self.y, self.w, rest_h);
        let bottom = Rect::new(self.x, self.y + rest_h, self.w, h);
        (bottom, rest) // 注意：为了符合直觉，通常我们要的是(切出来的, 剩下的)
        // 但底部切割时，"切出来的"在下面。这里返回 (切出的底部, 上面的剩余)
    }

    pub fn split_left(&self, width: f32) -> (Rect, Rect) {
        let w = width.min(self.w);
        let left = Rect::new(self.x, self.y, w, self.h);
        let rest = Rect::new(self.x + w, self.y, self.w - w, self.h);
        (left, rest)
    }

    pub fn split_right(&self, width: f32) -> (Rect, Rect) {
        let w = width.min(self.w);
        let rest_w = self.w - w;
        let rest = Rect::new(self.x, self.y, rest_w, self.h);
        let right = Rect::new(self.x + rest_w, self.y, w, self.h);
        (right, rest)
    }

    pub fn shrink(&self, amount: f32) -> Rect {
        // 如果缩没了，就返回0大小
        if self.w <= amount * 2.0 || self.h <= amount * 2.0 {
            return Rect::new(self.x, self.y, 0.0, 0.0);
        }
        Rect::new(
            self.x + amount,
            self.y + amount,
            self.w - amount * 2.0,
            self.h - amount * 2.0,
        )
    }
    pub fn center(&self, target_w: f32, target_h: f32) -> Rect {
        let new_x = self.x + (self.w - target_w) / 2.0;
        let new_y = self.y + (self.h - target_h) / 2.0;
        Rect::new(new_x, new_y, target_w, target_h)
    }
}