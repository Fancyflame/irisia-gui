use std::any::Any;

use crate::hook::utils::TraceCell;

pub struct Inner<T: ?Sized> {
    pub(super) callback_chain_storage: Box<dyn Any>,
    pub(super) value: TraceCell<T>,
}
