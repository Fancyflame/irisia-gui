use crate::{
    structure::activate::{ActivatedStructure, BareContentWrapper, Renderable, Structure, Visit},
    Result,
};

#[derive(Clone, Copy)]
pub struct EmptyStructure;

impl Structure for EmptyStructure {
    type Activated = EmptyStructure;
    fn activate(self, _: &mut (), _: &BareContentWrapper) -> Self {
        EmptyStructure
    }
}

impl ActivatedStructure for EmptyStructure {
    type Cache = ();
    fn element_count(&self) -> usize {
        0
    }
}

impl<V> Visit<V> for EmptyStructure {
    fn visit_at(&self, _: usize, _: &mut V) {}
}

impl<L> Renderable<L> for EmptyStructure {
    fn render_at(
        self,
        _: usize,
        _: &mut Self::Cache,
        _: BareContentWrapper,
        _: &mut L,
    ) -> Result<()> {
        Ok(())
    }
}
