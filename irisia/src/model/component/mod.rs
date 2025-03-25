use super::VModel;

pub mod field;
pub mod vmodel_builder;

pub trait Component {
    type Proxy: ComponentProxy;
}

pub trait ComponentProxy {}
