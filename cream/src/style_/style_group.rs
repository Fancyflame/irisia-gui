pub trait StyleGroup<'a, T> {
    fn apply_styles(&'a self, receiver: &mut T);
}

impl<'a, T, U> StyleGroup<'a, U> for &'a T
where
    T: StyleGroup<'a, U>,
{
    fn apply_styles(&'a self, receiver: &mut U) {
        (**self).apply_styles(receiver);
    }
}

pub struct StyleMixin<A, B> {
    branch: A,
    mixin: B,
}

impl<'a, A, B, T> StyleGroup<'a, T> for StyleMixin<A, B>
where
    A: StyleGroup<'a, T>,
    B: StyleGroup<'a, T>,
{
    fn apply_styles(&'a self, receiver: &mut T) {
        self.mixin.apply_styles(receiver);
        self.branch.apply_styles(receiver);
    }
}

impl<A, B> StyleMixin<A, B> {
    pub fn new(branch: A, mixin: B) -> Self {
        StyleMixin { branch, mixin }
    }
}

#[test]
fn test() {
    use crate::style_::StyleAccept;

    struct A;
    struct B;
    struct C;

    struct Style1 {
        a: A,
        c: C,
    }

    struct Style2 {
        b: B,
        c: C,
    }

    struct StyleRecv;

    impl<'a, T> StyleGroup<'a, T> for Style1
    where
        T: StyleAccept<'a, A> + StyleAccept<'a, C>,
    {
        fn apply_styles(&'a self, receiver: &mut T) {
            receiver.append_borrowed_style(&self.a);
            receiver.append_borrowed_style(&self.c);
        }
    }

    impl<'a, T> StyleGroup<'a, T> for Style2
    where
        T: StyleAccept<'a, B> + StyleAccept<'a, C>,
    {
        fn apply_styles(&'a self, receiver: &mut T) {
            receiver.append_borrowed_style(&self.b);
            receiver.append_borrowed_style(&self.c);
        }
    }

    macro_rules! impl_style_acpt {
        ($($T:ty),*) => {
            $(
                impl<'a> StyleAccept<'a, $T> for StyleRecv {
                    fn append_borrowed_style(&mut self, _: &'a $T) {}
                    fn append_sliced_style(&mut self, _: &'a [$T]) {}
                }
            )*
        };
    }

    impl_style_acpt!(A, B, C);

    let s1 = Style1 { a: A, c: C };
    let s2 = Style2 { b: B, c: C };
    let mix = StyleMixin::new(s1, &s2);

    mix.apply_styles(&mut StyleRecv);
}
