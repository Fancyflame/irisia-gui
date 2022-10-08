use crate::map_rc::MapRc;

use super::{DataSource, DepNode, Watchable};

pub struct ConstantValue<D>(D);

impl<D> Watchable<D> for ConstantValue<D> {
    fn get<'a>(&'a self) -> DataSource<D> {
        (&self.0).into()
    }

    fn subscribe(&self, _: &MapRc<dyn DepNode>) {}
}
