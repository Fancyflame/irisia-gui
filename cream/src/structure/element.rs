use anyhow::Result;
use skia_safe::Canvas;

pub trait Element {
    type Style: Eq;
    type AcceptChildren: ?Sized;
    fn render(
        &mut self,
        canvas: &mut Canvas,
        style: &Self::Style,
        children: &[&Self::AcceptChildren],
    ) -> Result<()>;
}

pub trait UpdateAndCreate<A>: Element + Sized {
    fn create(args: A) -> Result<Self>;

    /// If it needs to be redraw, return true, otherwise return false.
    fn update(&mut self, args: A) -> Result<bool>;
}
