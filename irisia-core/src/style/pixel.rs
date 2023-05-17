use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pixel(pub f32);

impl Pixel {
    pub fn to_physical(self) -> f32 {
        self.0
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
