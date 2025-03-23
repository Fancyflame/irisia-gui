use crate::{
    prim_element::EMCreateCtx,
    prim_element::{Element, GetElement},
};

use super::tools::DirtyPoints;

pub mod branch;
mod execute;
pub mod repeat;
pub mod slot;
pub mod tuple;

pub trait VModel {
    const EXECUTE_POINTS: usize;
    type Storage: Model;

    fn create(self, dp: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage;

    fn update(self, storage: &mut Self::Storage, dirty_points: &mut DirtyPoints, ctx: &EMCreateCtx);
}

pub trait Model: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element));
}

pub trait VNode: VModel<Storage: GetElement> {}

impl<T> VNode for T where T: VModel<Storage: GetElement> {}
