use iter::{ModelMapper, VisitModel};

use crate::{
    el_model::{EMCreateCtx, ElementModel},
    ElementInterfaces,
};

pub mod branch;
pub mod iter;
pub mod once;
pub mod reactive;
pub mod repeat;
pub mod tuple;

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

pub trait DesiredVModel<M: ModelMapper>: VModel<Storage: VisitModel<M>> {}

impl<M, T> DesiredVModel<M> for T
where
    M: ModelMapper,
    T: VModel<Storage: VisitModel<M>>,
{
}

pub trait RootDesiredModel<M: ModelMapper>:
    DesiredVModel<M, Storage = ElementModel<Self::RootEl, Self::RootCp>>
{
    type RootEl: ElementInterfaces;
    type RootCp;
}

impl<M, T, El, Cp> RootDesiredModel<M> for T
where
    M: ModelMapper,
    El: ElementInterfaces,
    Self: VModel<Storage = ElementModel<El, Cp>>,
    Self::Storage: VisitModel<M>,
{
    type RootEl = El;
    type RootCp = Cp;
}
