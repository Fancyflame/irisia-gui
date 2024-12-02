use iter::{ModelMapper, VisitModel};

use crate::el_model::EMCreateCtx;

pub mod cond;
pub mod iter;
pub mod optioned;
pub mod repeat;

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
