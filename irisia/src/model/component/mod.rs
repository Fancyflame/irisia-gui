pub use super::tools::DepManager;

pub mod child_box;
pub mod field;
pub mod vmodel_builder;

pub trait Component {
    type Proxy: ComponentProxy;
}

pub trait ComponentProxy {}
