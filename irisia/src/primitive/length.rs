use core::str;
use std::{
    fmt::{Debug, Formatter},
    io::{BufWriter, Write as _},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::{el_model::ElementAccess, primitive::Region};

macro_rules! create_length {
    {$($name:ident $short:ident,)*} => {
        #[derive(Clone, Copy, PartialEq, PartialOrd)]
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

            #[inline]
            fn debug_fields(&self) -> impl Iterator<Item = (f32, &'static str)> {
                [
                    $((self.$name, stringify!($short)),)*
                ].into_iter()
            }
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

        let (ew, eh) = match access.draw_region() {
            Some(Region {
                left_top,
                right_bottom,
            }) => (
                (right_bottom.0 - left_top.0) / 100.0,
                (right_bottom.1 - left_top.1) / 100.0,
            ),
            None => (0.0, 0.0),
        };

        let viewport_size = window.inner_size();
        let vw = viewport_size.width as f32 / 100.0;
        let vh = viewport_size.height as f32 / 100.0;
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

impl Debug for Length {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        enum State {
            Zero,
            Once { value: f32, unit: &'static str },
            Multiple,
        }

        let mut state = State::Zero;

        for (value, unit) in self.debug_fields() {
            if value == 0.0 {
                continue;
            }

            match state {
                State::Zero => {
                    state = State::Once { value, unit };
                    continue;
                }
                State::Once { value, unit } => {
                    write!(f, "(")?;
                    fmt_value(f, value, unit)?;
                    state = State::Multiple;
                }
                State::Multiple => {}
            }

            write!(f, " + ")?;
            fmt_value(f, value, unit)?;
        }

        match state {
            State::Zero => write!(f, "0px"),
            State::Once { value, unit } => fmt_value(f, value, unit),
            State::Multiple => {
                write!(f, ")")
            }
        }
    }
}

fn fmt_value(out_buf: &mut Formatter, value: f32, unit: &str) -> std::fmt::Result {
    let mut byte_buf = [0u8; 32];
    let mut fmt_buf = BufWriter::new(&mut byte_buf as &mut [u8]);
    write!(&mut fmt_buf, "{value:.2}").unwrap();
    let s = str::from_utf8(fmt_buf.buffer())
        .unwrap()
        .trim_end_matches('0')
        .trim_end_matches('.');
    write!(out_buf, "{s}{unit}")
}
