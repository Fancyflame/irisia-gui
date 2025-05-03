macro_rules! impl_two_dimensions {
    ($Type:ident $x:ident $y:ident) => {
        impl<T> $Type<T> {
            pub fn map<F, R>(self, f: F) -> $Type<R>
            where
                F: Fn(T) -> R,
            {
                $Type {
                    $x: f(self.$x),
                    $y: f(self.$y),
                }
            }
        }

        impl<T, Rhs> std::ops::Add<$Type<Rhs>> for $Type<T>
        where
            T: std::ops::Add<Rhs>,
        {
            type Output = $Type<T::Output>;
            #[inline]
            fn add(self, rhs: $Type<Rhs>) -> Self::Output {
                $Type {
                    $x: self.$x + rhs.$x,
                    $y: self.$y + rhs.$y,
                }
            }
        }

        impl<T, Rhs> std::ops::AddAssign<$Type<Rhs>> for $Type<T>
        where
            T: std::ops::AddAssign<Rhs>,
        {
            #[inline]
            fn add_assign(&mut self, rhs: $Type<Rhs>) {
                self.$x += rhs.$x;
                self.$y += rhs.$y;
            }
        }

        impl<T, Rhs> std::ops::Sub<$Type<Rhs>> for $Type<T>
        where
            T: std::ops::Sub<Rhs>,
        {
            type Output = $Type<T::Output>;
            #[inline]
            fn sub(self, rhs: $Type<Rhs>) -> Self::Output {
                $Type {
                    $x: self.$x - rhs.$x,
                    $y: self.$y - rhs.$y,
                }
            }
        }

        impl<T, Rhs> std::ops::SubAssign<$Type<Rhs>> for $Type<T>
        where
            T: std::ops::SubAssign<Rhs>,
        {
            #[inline]
            fn sub_assign(&mut self, rhs: $Type<Rhs>) {
                self.$x -= rhs.$x;
                self.$y -= rhs.$y;
            }
        }

        impl<T> From<(T, T)> for $Type<T> {
            #[inline]
            fn from(tuple: (T, T)) -> Self {
                $Type {
                    $x: tuple.0,
                    $y: tuple.1,
                }
            }
        }

        impl<T> From<$Type<T>> for (T, T) {
            #[inline]
            fn from(v: $Type<T>) -> Self {
                (v.$x, v.$y)
            }
        }
    };
}
