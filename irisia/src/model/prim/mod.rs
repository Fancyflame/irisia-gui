use crate::{
    hook::{reactive::Reactive, Signal},
    prim_element::Element,
};

use super::{EleModel, Model};

pub use self::{block::Block, text::Text};
pub use crate::prim_element::block::layout::DefaultLayouter;
pub(crate) use block::BlockModel;

mod block;
// mod image;
mod text;

impl<T> EleModel for Reactive<T>
where
    T: EleModel,
{
    fn get_element(&self) -> Element {
        self.read().get_element()
    }
}

impl<T> Model for Reactive<T>
where
    T: Model,
{
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        self.read().visit(f);
    }
}

struct PrimitiveVnodeWrapper<T>(T);

fn read_or_default<T: Clone>(signal: &Option<Signal<T>>, default: T) -> T {
    match signal {
        Some(sig) => sig.read().clone(),
        None => default,
    }
}

fn panic_when_call_unreachable() -> ! {
    panic!(
        "don't use primitive v-model directly, please use them as components through `build` macro"
    );
}
