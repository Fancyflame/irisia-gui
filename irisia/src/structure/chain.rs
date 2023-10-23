use super::{VisitBy, VisitLen, VisitMutBy};
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

impl<A, B> VisitLen for Chain<A, B>
where
    A: VisitLen,
    B: VisitLen,
{
    fn len(&self) -> usize {
        self.former.len() + self.latter.len()
    }
}

impl<A, B, V> VisitBy<V> for Chain<A, B>
where
    A: VisitBy<V>,
    B: VisitBy<V>,
{
    fn visit(&self, visitor: &mut V) -> Result<()> {
        self.former.visit(visitor)?;
        self.latter.visit(visitor)
    }
}

impl<A, B, V> VisitMutBy<V> for Chain<A, B>
where
    A: VisitMutBy<V>,
    B: VisitMutBy<V>,
{
    fn visit_mut(&mut self, visitor: &mut V) -> Result<()> {
        self.former.visit_mut(visitor)?;
        self.latter.visit_mut(visitor)
    }
}
