use crate::{Handle, hook::watcher::WatcherList, prim_element::Element};

use super::{EleModel, Model};

pub use self::{block::Block, text::Text};
pub use crate::prim_element::block::layout::DefaultLayouter;
pub(crate) use block::SubmitChildren;

mod block;
// mod image;
mod text;

struct PrimitiveVnodeWrapper<T>(T);

pub struct PrimitiveModel<T> {
    model: Handle<T>,
    _watcher_list: WatcherList,
}

impl<T, Cd> Model<Cd> for PrimitiveModel<T>
where
    T: Model<Cd>,
{
    fn visit(&self, f: &mut dyn FnMut(Element, Cd)) {
        self.model.borrow().visit(f);
    }
}

impl<T, Cd> EleModel<Cd> for PrimitiveModel<T>
where
    T: EleModel<Cd>,
{
    fn get_element(&self) -> (Element, Cd) {
        self.model.borrow().get_element()
    }
}

fn panic_when_call_unreachable() -> ! {
    panic!(
        "don't use primitive v-model directly, please use them as components through `build` macro"
    );
}
