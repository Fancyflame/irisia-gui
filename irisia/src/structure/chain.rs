use super::{VisitBy, VisitOn};
use crate::Result;

pub struct Chain<A, B>(pub A, pub B);

impl<A, B> VisitBy for Chain<A, B>
where
    A: VisitBy,
    B: VisitBy,
{
    fn visit_by<V: VisitOn>(&self, visitor: &mut V) -> Result<()> {
        self.0.visit_by(visitor)?;
        self.1.visit_by(visitor)
    }

    fn len(&self) -> usize {
        self.0.len() + self.1.len()
    }
}
