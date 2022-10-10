use crate::map_rc::MapRc;

use super::{DataSource, DepNode, Watchable};

pub struct ConstantValue<D>(D);

impl<D> ConstantValue<D> {
    pub fn new(value: D) -> Self {
        ConstantValue(value)
    }
}

impl<D> Watchable for ConstantValue<D> {
    type Data = D;

    fn get(&self) -> DataSource<D> {
        (&self.0).into()
    }

    fn subscribe(&self, _: &MapRc<dyn DepNode>) {}
}
