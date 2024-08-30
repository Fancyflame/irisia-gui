use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::el_model::ElementAccess;

macro_rules! create_length {
    {$($name:ident $short:ident,)*} => {
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct Length {
            $(pub $name: f32,)*
        }

        impl Length {
            $(
                #[doc = concat!("create a length with given value in unit `", stringify!($short), "`")]
                pub const fn $short(value: f32) -> Self {
                    Self {
                        $name: value,
                        ..Self::zero()
                    }
                }
            )*

            /// Create a zero length.
            pub const fn zero() -> Self {
                Self {
                    $($name: 0.0,)*
                }
            }

            pub /*const*/ fn add(mut self, rhs: Self) -> Self {
                $(self.$name += rhs.$name;)*
                self
            }

            pub /*const*/ fn mul(mut self, rhs: f32) -> Self {
                $(self.$name *= rhs;)*
                self
            }

            /*fn fields(&mut self) -> impl Iterator<Item = &mut f32> {
                [ $(&mut self.$name,)* ].into_iter()
            }*/
        }
    };
}

create_length! {
    pixel           px,
    viewport_width  vw,
    viewport_height vh,
    viewport_min    vmin,
    viewport_max    vmax,
    parent_width    pw,
    parent_height   ph,
    parent_min      pmin,
    parent_max      pmax,
}

impl Length {
    /// Convert the relative length to physical length of the screen with a window.
    /// The resolved value can be used directly to draw something.
    #[inline]
    pub fn to_resolved(&self, access: ElementAccess) -> f32 {
        let window = access.global_content().window();
        let (draw_start, draw_end) = access.draw_region();

        let viewport_size = window.inner_size();
        let vw = viewport_size.width as f32;
        let vh = viewport_size.height as f32;
        let ew = draw_end.0 - draw_start.0;
        let eh = draw_end.1 - draw_start.1;
        let dpi = window.scale_factor() as f32;

        self.pixel * dpi
            + self.viewport_width * vw
            + self.viewport_height * vh
            + self.viewport_min * vw.min(vh)
            + self.viewport_max * vw.max(vh)
            + self.parent_width * ew
            + self.parent_height * eh
            + self.parent_min * ew.min(eh)
            + self.parent_max * ew.max(eh)
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::zero()
    }
}

impl Mul<f32> for Length {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        self.mul(rhs)
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
    fn div(self, rhs: f32) -> Self::Output {
        self * (1.0 / rhs)
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
    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
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
