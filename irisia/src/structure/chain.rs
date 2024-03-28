use crate::Element;

use super::{StructureUpdater, VisitBy};

pub struct Chain<A, B>(pub A, pub B);

impl<A, B> VisitBy for Chain<A, B>
where
    A: VisitBy,
    B: VisitBy,
{
    fn iter(&self) -> impl Iterator<Item = &dyn Element> {
        self.0.iter().chain(self.1.iter())
    }

    fn visit_mut(
        &mut self,
        mut f: impl FnMut(&mut dyn Element) -> crate::Result<()>,
    ) -> crate::Result<()> {
        self.0.visit_mut(&mut f)?;
        self.1.visit_mut(f)
    }

    fn len(&self) -> usize {
        self.0.len() + self.1.len()
    }
}

impl<A, B> StructureUpdater for Chain<A, B>
where
    A: StructureUpdater,
    B: StructureUpdater,
{
    type Target = Chain<A::Target, B::Target>;

    fn create(self, ctx: &crate::dom::EMCreateCtx) -> Self::Target {
        Chain(self.0.create(ctx), self.1.create(ctx))
    }

    fn update(self, target: &mut Self::Target, ctx: &crate::dom::EMCreateCtx) {
        self.0.update(&mut target.0, ctx);
        self.1.update(&mut target.1, ctx);
    }
}
