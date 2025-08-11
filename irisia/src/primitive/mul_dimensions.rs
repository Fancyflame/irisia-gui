macro_rules! impl_mul_dimensions {
    ($Type:ident $($dim: ident)*) => {
        impl<T> $Type<T> {
            pub const fn all(value: T) -> Self
            where
                T: Copy
            {
                $Type {
                    $($dim: value,)*
                }
            }

            pub fn map<F, R>(self, f: F) -> $Type<R>
            where
                F: Fn(T) -> R,
            {
                $Type {
                    $($dim: f(self.$dim),)*
                }
            }

            pub fn map_with<F, Rhs, R>(self, rhs: $Type<Rhs>, f: F) -> $Type<R>
            where
                F: Fn(T, Rhs) -> R,
            {
                $Type {
                    $($dim: f(self.$dim, rhs.$dim),)*
                }
            }

            pub fn as_ref(&self) -> $Type<&T> {
                $Type {
                    $($dim: &self.$dim,)*
                }
            }

            pub fn as_mut(&mut self) -> $Type<&mut T> {
                $Type {
                    $($dim: &mut self.$dim,)*
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
                self.map_with(rhs, T::add)
            }
        }

        impl<T, Rhs> std::ops::AddAssign<$Type<Rhs>> for $Type<T>
        where
            T: std::ops::AddAssign<Rhs>,
        {
            #[inline]
            fn add_assign(&mut self, rhs: $Type<Rhs>) {
                $(self.$dim += rhs.$dim;)*
            }
        }

        impl<T, Rhs> std::ops::Sub<$Type<Rhs>> for $Type<T>
        where
            T: std::ops::Sub<Rhs>,
        {
            type Output = $Type<T::Output>;
            #[inline]
            fn sub(self, rhs: $Type<Rhs>) -> Self::Output {
                self.map_with(rhs, T::sub)
            }
        }

        impl<T, Rhs> std::ops::SubAssign<$Type<Rhs>> for $Type<T>
        where
            T: std::ops::SubAssign<Rhs>,
        {
            #[inline]
            fn sub_assign(&mut self, rhs: $Type<Rhs>) {
                $(self.$dim -= rhs.$dim;)*
            }
        }

        impl<T> std::ops::Neg for $Type<T>
        where
            T: std::ops::Neg,
        {
            type Output = $Type<T::Output>;
            fn neg(self) -> Self::Output {
                self.map(T::neg)
            }
        }
    };
}
