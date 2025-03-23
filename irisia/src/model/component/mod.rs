use super::VModel;

pub mod field;
pub mod vmodel_builder;

pub trait Component {
    type Proxy: VModel;
}

pub trait ComponentProxy {
    type Input<'a>;
}
