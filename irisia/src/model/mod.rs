use iter::{ModelMapper, VisitModel};

use crate::{
    el_model::{EMCreateCtx, ElementModel},
    ElementInterfaces,
};

pub mod branch;
pub mod iter;
pub mod reactive;
pub mod repeat;
pub mod tuple;
pub mod unit;

pub trait VModel {
    type Storage: 'static;

    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx);
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage;
}

impl<T> VModel for &T
where
    T: VModel,
{
    type Storage = T::Storage;
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        (*self).create(ctx)
    }
    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        (*self).update(storage, ctx);
    }
}

pub trait DesiredVModel<M: ModelMapper>: VModel<Storage: VisitModel<M>> + 'static {}

impl<M, T> DesiredVModel<M> for T
where
    M: ModelMapper,
    T: VModel<Storage: VisitModel<M>> + 'static,
{
}

pub trait RootDesiredModel<M: ModelMapper>:
    DesiredVModel<M, Storage = ElementModel<Self::RootEl, Self::RootCp, Self::RootSlt>>
{
    type RootEl: ElementInterfaces;
    type RootCp;
    type RootSlt;
}

impl<M, T, El, Cp, Slt> RootDesiredModel<M> for T
where
    M: ModelMapper,
    El: ElementInterfaces,
    Self: VModel<Storage = ElementModel<El, Cp, Slt>> + 'static,
    Self::Storage: VisitModel<M>,
{
    type RootEl = El;
    type RootCp = Cp;
    type RootSlt = Slt;
}
