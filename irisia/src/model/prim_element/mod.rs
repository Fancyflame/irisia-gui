use crate::hook::reactive::Reactive;

use super::Model;

pub mod block;

impl<T> Model for Reactive<T>
where
    T: Model,
{
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.read().visit(f);
    }
}

struct PrimitiveVModelWrapper<T>(T);
