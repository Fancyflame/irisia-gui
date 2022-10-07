pub trait StyleVariant<'a>: Sized {
    type InputStyle;
    fn from_borrowed_style(from: &'a Self::InputStyle) -> Self;
    fn from_sliced_style(from: &'a [Self::InputStyle]) -> Self;
}

pub struct ImAStyle<T>(pub T);

impl<'a, T> StyleVariant<'a> for &'a [T] {
    type InputStyle = T;
    fn from_borrowed_style(from: &'a T) -> Self {
        std::array::from_ref(from)
    }
    fn from_sliced_style(from: &'a [T]) -> Self {
        from
    }
}

impl<'a, T> StyleVariant<'a> for &'a T
where
    &'a T: Default,
{
    type InputStyle = T;
    fn from_borrowed_style(from: &'a Self::InputStyle) -> Self {
        from
    }
    fn from_sliced_style(from: &'a [Self::InputStyle]) -> Self {
        match from.first() {
            Some(t) => t,
            None => {
                if cfg!(debug_assertions) {
                    panic!("The slice is empty but at least one style definition is required");
                } else {
                    Default::default()
                }
            }
        }
    }
}

impl<'a, T> StyleVariant<'a> for Option<&'a T> {
    type InputStyle = T;
    fn from_borrowed_style(from: &'a T) -> Self {
        Some(from)
    }
    fn from_sliced_style(from: &'a [T]) -> Self {
        from.first()
    }
}

impl<'a, T> StyleVariant<'a> for Option<&'a [T]> {
    type InputStyle = T;
    fn from_borrowed_style(from: &'a T) -> Self {
        Some(<_>::from_borrowed_style(from))
    }
    fn from_sliced_style(from: &'a [T]) -> Self {
        Some(from)
    }
}

impl<'a, T: Clone + Default> StyleVariant<'a> for ImAStyle<T> {
    type InputStyle = T;
    fn from_borrowed_style(from: &'a T) -> Self {
        ImAStyle(from.clone())
    }
    fn from_sliced_style(from: &'a [T]) -> Self {
        match from.first() {
            Some(t) => ImAStyle(t.clone()),
            None => {
                if cfg!(debug_assertions) {
                    panic!("The slice is empty but at least one style definition is required");
                } else {
                    ImAStyle(T::default())
                }
            }
        }
    }
}
