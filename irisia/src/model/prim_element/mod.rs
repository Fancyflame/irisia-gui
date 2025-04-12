use crate::hook::{reactive::Reactive, Signal};

use super::Model;

pub use self::{
    block::{Block, DEFAULT_LAYOUT_FN},
    rect::Rect,
    text::Text,
};
pub(crate) use block::BlockModel;

mod block;
mod rect;
mod text;

impl<T> Model for Reactive<T>
where
    T: Model,
{
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.read().visit(f);
    }
}

struct PrimitiveVModelWrapper<T>(T);

fn read_or_default<T: Clone>(signal: &Option<Signal<T>>, default: T) -> T {
    match signal {
        Some(sig) => sig.read().clone(),
        None => default,
    }
}
