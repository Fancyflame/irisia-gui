use crate::el_model::EMCreateCtx;

pub mod branch;
pub mod iter;
pub mod reactive;
pub mod repeat;
pub mod tuple;
pub mod unit;

pub trait VModel {
    type Storage: 'static;

    fn update(self, storage: &mut Self::Storage, ctx: &EMCreateCtx);
    fn create(self, ctx: &EMCreateCtx) -> Self::Storage;
}
