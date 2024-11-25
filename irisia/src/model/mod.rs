use iter::{IterModel, ModelMapper};

use crate::el_model::EMCreateCtx;

pub mod basic;
pub mod iter;

pub trait ModelCreateFn<M: ModelMapper>: FnOnce(&EMCreateCtx) -> Self::Model {
    type Model: IterModel<M> + 'static;
}

impl<F, R, M> ModelCreateFn<M> for F
where
    F: FnOnce(&EMCreateCtx) -> R,
    R: IterModel<M> + 'static,
    M: ModelMapper,
{
    type Model = R;
}
