// #[rustfmt::skip]

pub use crate::model::component::{
    definition::{
        Definition, // trait Definition
        DirectAssign,
        proxy_signal::helper::check_eq as new_proxy_signal, // fn new_proxy_signal
    },
    property, // mod property
};
