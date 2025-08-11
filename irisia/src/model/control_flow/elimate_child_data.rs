use std::marker::PhantomData;

use crate::{
    model::{Model, ModelCreateCtx, VModel},
    prim_element::Element,
};

pub struct ElimateChildData<T, _Cd = ()> {
    _phantom_child_data: PhantomData<_Cd>,
    model: T,
}

impl<T, _Cd> ElimateChildData<T, _Cd> {
    pub fn new(model: T) -> Self
    where
        T: VModel<_Cd>,
    {
        Self {
            _phantom_child_data: PhantomData,
            model,
        }
    }
}

impl<T, Cd, _Cd> VModel<Cd> for ElimateChildData<T, _Cd>
where
    T: VModel<_Cd>,
    Cd: Default,
    _Cd: 'static,
{
    type Storage = ElimateChildData<T::Storage, _Cd>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        ElimateChildData {
            _phantom_child_data: PhantomData,
            model: self.model.create(ctx),
        }
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        self.model.update(&mut storage.model, ctx)
    }
}

impl<T, Cd, _Cd> Model<Cd> for ElimateChildData<T, _Cd>
where
    T: Model<_Cd>,
    Cd: Default,
    _Cd: 'static,
{
    fn visit(&self, f: &mut dyn FnMut(Element, Cd)) {
        self.model.visit(&mut |el, _| f(el, Cd::default()))
    }
}
