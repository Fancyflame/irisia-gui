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

pub trait Model<Data = ()>: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element, Data));
}

pub trait UnitModel<Data = ()>: Model<Data> {
    fn get_element(&self) -> (Element, Data);
}

pub trait VNode<Data = ()>: VModel<Data, Storage: UnitModel<Data>> {}
impl<Data, T> VNode<Data> for T where T: VModel<Data, Storage: UnitModel<Data>> + ?Sized {}

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
