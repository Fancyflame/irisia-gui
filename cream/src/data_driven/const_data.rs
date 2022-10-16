use std::rc::Rc;

use super::{DataSource, DepNode, Watchable};

pub struct ConstData<D>(D);

impl<D> ConstData<D> {
    pub fn new(value: D) -> Rc<Self> {
        Rc::new(ConstData(value))
    }
}

impl<D> Watchable for ConstData<D> {
    type Data = D;

    fn get(&self) -> DataSource<D> {
        (&self.0).into()
    }

    fn subscribe(&self, _: &Rc<dyn DepNode>) {}
}
