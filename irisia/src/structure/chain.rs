use super::{VisitBy, VisitOn};
use crate::Result;

pub struct Chain<A, B> {
    pub former: A,
    pub latter: B,
}

impl<A, B> Chain<A, B> {
    pub fn new(former: A, latter: B) -> Self {
        Chain { former, latter }
    }
}

impl<A, B> VisitBy for Chain<A, B>
where
    A: VisitBy,
    B: VisitBy,
{
    fn visit_by<V: VisitOn>(&self, visitor: &mut V) -> Result<()> {
        self.former.visit_by(visitor)?;
        self.latter.visit_by(visitor)
    }

    fn len(&self) -> usize {
        self.former.len() + self.latter.len()
    }
}
