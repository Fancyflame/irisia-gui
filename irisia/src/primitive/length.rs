use std::ops::{
    Add, AddAssign, Bound, Div, DivAssign, Mul, MulAssign, Neg, RangeBounds, Sub, SubAssign,
};

use irisia_backend::{winit::dpi::PhysicalSize, WinitWindow};

use crate::el_model::ElementAccess;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
struct SimpleLength {
    pixel: f32,
    viewport_width: f32,
    viewport_height: f32,
    viewport_min: f32,
    viewport_max: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length {
    value: SimpleLength,
    lower_limit: Option<SimpleLength>,
    upper_limit: Option<SimpleLength>,
}

macro_rules! create_length {
    {$($name:ident $short:ident,)*} => {
        $(
            #[doc = concat!("create a length with given value in unit `", stringify!($short), "`")]
            pub const fn $short(value: f32) -> Self {
                let mut this = Self::zero();
                this.value.$name = value;
                this
            }
        )*
    };
}

impl SimpleLength {
    fn to_resolved(&self, window: &WinitWindow) -> f32 {
        let PhysicalSize { width, height } = window.inner_size();
        let w = width as f32;
        let h = height as f32;
        let dpi = window.scale_factor() as f32;

        self.pixel * dpi
            + self.viewport_width * w
            + self.viewport_height * h
            + self.viewport_min * w.min(h)
            + self.viewport_max * w.max(h)
    }

    fn fields(&mut self) -> impl Iterator<Item = &mut f32> {
        [
            &mut self.pixel,
            &mut self.viewport_width,
            &mut self.viewport_height,
            &mut self.viewport_min,
            &mut self.viewport_max,
        ]
        .into_iter()
    }
}

impl Length {
    /// Create a zero length.
    pub const fn zero() -> Self {
        Self {
            value: SimpleLength {
                pixel: 0.0,
                viewport_width: 0.0,
                viewport_height: 0.0,
                viewport_min: 0.0,
                viewport_max: 0.0,
            },
            lower_limit: None,
            upper_limit: None,
        }
    }

    create_length! {
        pixel           px,
        viewport_width  vw,
        viewport_height vh,
        viewport_min    vmin,
        viewport_max    vmax,
    }

    /// Equivalent to
    /// `self.to_resolved_with_window(access.global_content().window())`
    #[inline]
    pub fn to_resolved(&self, access: &ElementAccess) -> f32 {
        self.to_resolved_with_window(access.global_content().window())
    }

    /// Convert the length to a resolved value with a window.
    /// The resolved value can be used directly to draw something.
    #[inline]
    pub fn to_resolved_with_window(&self, window: &WinitWindow) -> f32 {
        let mut value = self.value.to_resolved(window);

        if let Some(lower) = self.lower_limit {
            value = value.max(lower.to_resolved(window));
        }

        if let Some(upper) = self.upper_limit {
            value = value.min(upper.to_resolved(window));
        }

        value
    }

    /// Set the lower and upper limit of the length.
    /// The length will be clamped to the range.
    pub fn limit<R>(mut self, range: R) -> Self
    where
        R: RangeBounds<Self>,
    {
        fn get_bound(bound: Bound<&Length>) -> Option<SimpleLength> {
            match bound {
                Bound::Included(v) | Bound::Excluded(v) => {
                    debug_assert!(!v.has_limit(), "range bounds cannot have limit");
                    Some(v.value)
                }
                Bound::Unbounded => None,
            }
        }
        self.lower_limit = get_bound(range.start_bound());
        self.upper_limit = get_bound(range.end_bound());
        self
    }

    /// Check if the length has lower or upper limit.
    #[inline]
    pub fn has_limit(&self) -> bool {
        self.lower_limit.is_some() || self.upper_limit.is_some()
    }
}

impl Mul<f32> for Length {
    type Output = Self;

    #[inline]
    fn mul(mut self, rhs: f32) -> Self::Output {
        self.value.fields().for_each(|v| *v *= rhs);
        self
    }
}

impl MulAssign<f32> for Length {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Div<f32> for Length {
    type Output = Self;

    #[inline]
    fn div(mut self, rhs: f32) -> Self::Output {
        self.value.fields().for_each(|v| *v /= rhs);
        self
    }
}

impl DivAssign<f32> for Length {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl Neg for Length {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self * -1.0
    }
}

impl Add<Self> for Length {
    type Output = Self;

    #[inline]
    fn add(mut self, mut rhs: Self) -> Self::Output {
        self.value
            .fields()
            .zip(rhs.value.fields())
            .for_each(|(a, b)| *a += *b);
        self
    }
}

impl AddAssign<Self> for Length {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub<Self> for Length {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl SubAssign<Self> for Length {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
