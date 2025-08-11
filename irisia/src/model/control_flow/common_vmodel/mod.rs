pub use {
    vmodel::{BoxedModel, CommonVModel},
    vnode::{BoxedNode, CommonVNode},
};

pub type DynVModel<T> = dyn CommonVModel<T> + 'static;
pub type DynVNode<T> = dyn CommonVNode<T> + 'static;

mod vmodel;
mod vnode;
