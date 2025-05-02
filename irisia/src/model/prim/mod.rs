use crate::{
    hook::{watcher::WatcherList, Signal},
    prim_element::Element,
    Handle,
};

use super::{EleModel, Model};

pub use self::{block::Block, text::Text};
pub use crate::prim_element::block::layout::DefaultLayouter;
pub(crate) use block::BlockModel;

mod block;
// mod image;
mod text;

struct PrimitiveVnodeWrapper<T>(T);

pub struct PrimitiveModel<T> {
    model: Handle<T>,
    _watcher_list: WatcherList,
}

impl<T: Model> Model for PrimitiveModel<T> {
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        self.model.borrow().visit(f);
    }
}

impl<T: EleModel> EleModel for PrimitiveModel<T> {
    fn get_element(&self) -> Element {
        self.model.borrow().get_element()
    }
}

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
