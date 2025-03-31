use std::any::Any;

use crate::prim_element::EMCreateCtx;

use crate::model::{tools::DirtyPoints, Model, VModel};

trait AnyModel: Model {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AnyModel for T
where
    T: Model,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct ChildBox(Box<dyn AnyModel>);

pub struct ChildBoxWrapper<T>(pub T);

impl<'a, T> VModel<'a> for ChildBoxWrapper<T>
where
    T: VModel<'a>,
{
    const EXECUTE_POINTS: usize = T::EXECUTE_POINTS;
    type Storage = ChildBox;

    fn create(self, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) -> Self::Storage {
        ChildBox(Box::new(self.0.create(dp, ctx)))
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) {
        match storage.0.as_any_mut().downcast_mut::<T::Storage>() {
            Some(inner_storage) => self.0.update(inner_storage, dp, ctx),
            None => {
                eprintln!(
                    "warning: type mismatch detected when using child box. create model instead."
                );
                *storage = self.create(dp, ctx);
            }
        }
    }
}

impl Model for ChildBox {
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.0.visit(f);
    }
}
