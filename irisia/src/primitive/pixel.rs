use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// `Pixel` is a logical length, difference from physical length.
///
/// Except drawing something on canvas, at everywhere should use
/// `Pixel` instead of primitive number to express a logical length.
#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pixel(pub f32);

const DPI: f32 = 1.0;

impl Pixel {
    pub fn to_physical(self) -> f32 {
        self.0 * DPI
    }

    pub fn from_physical(p: f32) -> Self {
        Pixel(p) / DPI
    }

    pub fn min(self, other: Self) -> Self {
        Pixel(self.0.min(other.0))
    }

    pub fn max(self, other: Self) -> Self {
        Pixel(self.0.max(other.0))
    }
}

impl Debug for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}px", self.0)
    }
}

macro_rules! impl_opr {
    ($var:ident,
        $($Trait:ident $TraitAssign:ident $Rhs:ident $fnname:ident $fnassign:ident $opr:tt $vexpr:expr,)*) => {
        $(
            impl $Trait<$Rhs> for Pixel {
                type Output = Self;
                fn $fnname(self, $var: $Rhs) -> Self {
                    Self(self.0 $opr $vexpr)
                }
            }

            impl $TraitAssign<$Rhs> for Pixel {
                fn $fnassign(&mut self, $var: $Rhs) {
                    self.0 = self.0 $opr $vexpr;
                }
            }
        )*
    };
}

impl_opr! {
    rhs,
    Add AddAssign Self add add_assign + rhs.0,
    Sub SubAssign Self sub sub_assign - rhs.0,
    Mul MulAssign f32 mul mul_assign * rhs,
    Div DivAssign f32 div div_assign / rhs,
}

impl Neg for Pixel {
    type Output = Self;
    fn neg(self) -> Self {
        Pixel(-self.0)
    }
}

impl From<f32> for Pixel {
    fn from(value: f32) -> Self {
        Pixel(value)
    }
}
