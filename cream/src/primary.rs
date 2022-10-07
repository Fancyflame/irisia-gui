use std::ops::{Add, Sub};

pub type Area = (Vec2, Vec2);
pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Vec2(pub u64, pub u64);

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vec2(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Vec2 {
    /// Absolutely greater than
    #[inline]
    pub fn abs_ge(self, other: Self) -> bool {
        self.0 >= other.0 && self.1 >= other.1
    }

    /// Absolutely less than
    #[inline]
    pub fn abs_le(self, other: Self) -> bool {
        self.0 <= other.0 && self.1 <= other.1
    }
}

impl From<(u64, u64)> for Vec2 {
    #[inline]
    fn from(f: (u64, u64)) -> Self {
        Vec2(f.0, f.1)
    }
}

impl From<Vec2> for (u64, u64) {
    fn from(v: Vec2) -> Self {
        (v.0, v.1)
    }
}

/*pub struct MyRect;

impl Widget for MyRect {
    fn draw(&self, canvas: &mut Canvas, size: RenderSize) {
        let rect = Rect::new(0.0, 0.0, size.0 as f32, size.1 as f32);
        let mut paint = Paint::default();
        paint.set_anti_alias(true).set_argb(255, 255, 255, 0);
        canvas.draw_rect(rect, &paint);
    }
}*/
