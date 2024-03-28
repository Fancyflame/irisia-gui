use super::{StructureUpdater, VisitBy};
use crate::{dom::EMCreateCtx, Element};

impl VisitBy for () {
    fn iter(&self) -> impl Iterator<Item = &dyn Element> {
        std::iter::empty()
    }

    fn visit_mut(
        &mut self,
        _: impl FnMut(&mut dyn crate::Element) -> crate::Result<()>,
    ) -> crate::Result<()> {
        Ok(())
    }

    fn len(&self) -> usize {
        0
    }

    fn is_empty(&self) -> bool {
        true
    }
}

impl StructureUpdater for () {
    type Target = ();

    fn create(self, _: &EMCreateCtx) -> Self::Target {}
    fn update(self, _: &mut Self::Target, _: &EMCreateCtx) {}
}
