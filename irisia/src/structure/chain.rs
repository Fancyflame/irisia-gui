use super::{StructureUpdateTo, VisitBy, VisitOn};
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

impl<A, B, const WD: usize> StructureUpdateTo<WD> for Chain<A, B>
where
    A: StructureUpdateTo<WD>,
    B: StructureUpdateTo<WD>,
{
    type Target = Chain<A::Target, B::Target>;
    const UPDATE_POINTS: u32 = A::UPDATE_POINTS + B::UPDATE_POINTS;

    fn create(self, mut info: super::Updating<WD>) -> Self::Target {
        Chain(
            self.0.create(info.inherit(0, false)),
            self.1.create(info.inherit(A::UPDATE_POINTS, false)),
        )
    }

    fn update(self, target: &mut Self::Target, mut info: super::Updating<WD>) {
        if info.no_update::<Self>() {
            return;
        }

        self.0.update(&mut target.0, info.inherit(0, false));
        self.1
            .update(&mut target.1, info.inherit(A::UPDATE_POINTS, false))
    }
}
