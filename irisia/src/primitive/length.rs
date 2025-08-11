use core::str;
use std::{
    fmt::{Debug, Formatter},
    io::{BufWriter, Write as _},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use super::size::Size;

#[derive(Clone, Copy, PartialEq)]
pub struct LengthStandard {
    pub global: LengthStandardGlobalPart,
    pub percentage_reference: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub struct LengthStandardGlobalPart {
    pub dpi: f32,
    pub viewport_size: Size<u32>,
}

impl LengthStandard {
    pub fn resolve(&self, length: Length) -> Option<f32> {
        match length {
            Length::Measured(m) => Some(m.to_resolved(self)),
            Length::Auto => None,
        }
    }

    pub(crate) fn resolve_fn(&self) -> impl Fn(Length) -> Option<f32> {
        |len| self.resolve(len)
    }

    pub fn set_percentage_reference(&self, r: f32) -> Self {
        Self {
            global: self.global,
            percentage_reference: r,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Length {
    #[default]
    Auto,
    Measured(MeasuredLength),
}

impl Length {
    pub const ZERO: Self = Length::Measured(MeasuredLength::ZERO);

    pub fn map<F>(self, f: F) -> Length
    where
        F: FnOnce(MeasuredLength) -> MeasuredLength,
    {
        match self {
            Length::Auto => Length::Auto,
            Length::Measured(m) => Length::Measured(f(m)),
        }
    }
}

macro_rules! create_length {
    {$($name:ident $UNIT:ident $short:ident,)*} => {
        #[derive(Clone, Copy, PartialEq)]
        pub struct MeasuredLength {
            $(pub $name: f32,)*
        }

        $(
            #[doc = stringify!(1 $UNIT)]
            pub const $UNIT: Length = Length::Measured(
                MeasuredLength {
                    $name: 1.0,
                    ..MeasuredLength::ZERO
                }
            );
        )*

        impl MeasuredLength {
            /// Create a zero length.
            pub const ZERO: Self = Self {
                $($name: 0.0,)*
            };

            pub const fn add(mut self, rhs: Self) -> Self {
                $(self.$name += rhs.$name;)*
                self
            }

            pub const fn mul(mut self, rhs: f32) -> Self {
                $(self.$name *= rhs;)*
                self
            }

            /*fn fields(&mut self) -> impl Iterator<Item = &mut f32> {
                [ $(&mut self.$name,)* ].into_iter()
            }*/

            #[inline]
            fn debug_fields(&self) -> impl Iterator<Item = (f32, &'static str)> + use<> {
                [
                    $((self.$name, stringify!($short)),)*
                ].into_iter()
            }
        }
    };
}

create_length! {
    pixel           PX   px,
    viewport_width  VW   vw,
    viewport_height VH   vh,
    viewport_min    VMIN vmin,
    viewport_max    VMAX vmax,
    percent         PCT  pct,
}

impl Default for MeasuredLength {
    fn default() -> Self {
        Self::ZERO
    }
}

impl MeasuredLength {
    /// Convert the relative length to physical length of the screen with a window.
    /// The resolved value can be used directly to draw something.
    pub fn to_resolved(&self, standard: &LengthStandard) -> f32 {
        let &LengthStandard {
            percentage_reference: parent_axis_len,
            global: LengthStandardGlobalPart { dpi, viewport_size },
        } = standard;

        let vw = viewport_size.width as f32;
        let vh = viewport_size.height as f32;

        self.pixel * dpi
            + self.viewport_width * vw
            + self.viewport_height * vh
            + self.viewport_min * vw.min(vh)
            + self.viewport_max * vw.max(vh)
            + self.percent * parent_axis_len
    }
}

impl Mul<f32> for Length {
    type Output = Length;

    fn mul(self, rhs: f32) -> Self::Output {
        self.map(|this| this.mul(rhs))
    }
}

impl Mul<Length> for i32 {
    type Output = Length;

    fn mul(self, rhs: Length) -> Self::Output {
        rhs.map(|rhs| rhs.mul(self as _))
    }
}

impl Mul<Length> for f32 {
    type Output = Length;

    fn mul(self, rhs: Length) -> Self::Output {
        rhs.map(|rhs| rhs.mul(self))
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
        match (self, rhs) {
            (Length::Measured(lhs), Length::Measured(rhs)) => Length::Measured(lhs.add(rhs)),
            (Length::Measured(v), _) | (_, Length::Measured(v)) => Length::Measured(v),
            (Length::Auto, Length::Auto) => Length::Auto,
        }
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
        let length = match self {
            Length::Auto => return write!(f, "auto"),
            Length::Measured(l) => l,
        };

        let mut is_first = true;

        for (value, unit) in length.debug_fields() {
            if value == 0.0 {
                continue;
            }

            if is_first {
                is_first = false;
            } else {
                write!(f, " + ")?;
            }

            fmt_value(f, value, unit)?;
        }

        if is_first { write!(f, "0px") } else { Ok(()) }
    }
}

fn fmt_value(out_buf: &mut Formatter, value: f32, unit: &str) -> std::fmt::Result {
    let mut byte_buf = [0u8; 32];
    let mut fmt_buf = BufWriter::new(&mut byte_buf as &mut [u8]);
    write!(&mut fmt_buf, "{value:.3}").unwrap();
    let s = str::from_utf8(fmt_buf.buffer())
        .unwrap()
        .trim_end_matches('0')
        .trim_end_matches('.');
    write!(out_buf, "{s}{unit}")
}
