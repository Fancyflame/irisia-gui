use anyhow::Result;
use skia_safe::Canvas;

use crate::{map_rc::MapRc, style::StyleTable};

pub trait Element {
    type AcceptChildren: ?Sized;
    fn render(
        &mut self,
        canvas: &mut Canvas,
        style: &StyleTable,
        children: &[MapRc<Self::AcceptChildren>],
    ) -> Result<()>;
}

// A: (MapRc<dyn Watchable<T0>>, MapRc<dyn Watchable<T1>>, ...)
pub trait UpdateAndCreate<A>: Element + Sized {
    fn create(args: A) -> Result<Self>;
}
