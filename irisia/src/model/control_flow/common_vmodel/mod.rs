pub use {
    vmodel::{BoxedModel, CommonVModel},
    vnode::{BoxedNode, CommonVNode},
};

pub type DynVModel<T> = dyn CommonVModel<CommonParentProps = T> + 'static;
pub type DynVNode<T> = dyn CommonVNode<CommonParentPropsNode = T> + 'static;

mod vmodel;
mod vnode;
