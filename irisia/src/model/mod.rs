use crate::{
    WeakHandle,
    model::{control_flow::elimate_child_data::ElimateChildData, prim::SubmitChildren},
    prim_element::{EMCreateCtx, Element},
};

pub use style::UseStyle;

pub mod component;
pub mod control_flow;
// pub mod map_parent_props;
pub mod prim;
pub mod style;

pub trait VModel<Cd> {
    type Storage: Model<Cd>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage;
    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx);

    // Provided

    fn elimate_child_data(self) -> ElimateChildData<Self, Cd>
    where
        Self: Sized,
    {
        ElimateChildData::new(self)
    }
}

pub trait Model<Cd>: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element, Cd));
}

#[derive(Clone)]
pub struct ModelCreateCtx {
    el_ctx: EMCreateCtx,
    parent: Option<WeakHandle<dyn SubmitChildren>>,
}

impl ModelCreateCtx {
    pub(crate) fn create_as_root(ctx: EMCreateCtx) -> Self {
        Self {
            el_ctx: ctx,
            parent: None,
        }
    }
}

/// VModel provides guaranteed only 1 element
pub trait VNode<Cd>: VModel<Cd, Storage: EleModel<Cd>> {}
impl<Cd, T> VNode<Cd> for T where T: VModel<Cd, Storage: EleModel<Cd>> {}

pub trait EleModel<Cd>: Model<Cd> {
    fn get_element(&self) -> (Element, Cd);
}
