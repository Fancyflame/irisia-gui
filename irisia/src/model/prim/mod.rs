use super::Model;
use crate::{
    Handle,
    hook::watcher::Watcher,
    model::{UnitAssertion, VisitModelFn},
};

pub use self::{block::Block, text::Text};
pub use crate::prim_element::block::layout::DefaultLayouter;
pub(crate) use block::SubmitChildren;

mod block;
// mod image;
mod text;

struct PrimitiveVnodeWrapper<T>(T);

pub struct PrimitiveModel<T> {
    model: Handle<T>,
    _watcher_list: Vec<Watcher>,
}

impl<T> Model for PrimitiveModel<T>
where
    T: Model,
{
    fn visit_raw(&self, f: VisitModelFn) {
        self.model.borrow().visit_raw(f);
    }
}

impl<T> UnitAssertion for PrimitiveModel<T> {}

fn panic_when_call_unreachable() -> ! {
    panic!(
        "don't use primitive v-model directly, please use them as components through `build` macro"
    );
}
