use iter::{ModelMapper, VisitModel};

use crate::el_model::EMCreateCtx;

pub mod branch;
pub mod iter;
pub mod once;
pub mod reactive;
pub mod repeat;
pub mod tuple;

pub trait ModelCreateFn<M: ModelMapper>: Fn(&EMCreateCtx) -> Self::Model {
    type Model: VisitModel<M> + 'static;
}

impl<F, R, M> ModelCreateFn<M> for F
where
    F: Fn(&EMCreateCtx) -> R,
    R: VisitModel<M> + 'static,
    M: ModelMapper,
{
    type Model = R;
}

pub trait VModel {
    type Storage: 'static;

    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx);
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage;
}
