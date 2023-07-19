use crate::{
    element::render_content::BareContent,
    structure::activate::{
        ActivateUpdateArguments, ActivatedStructure, Renderable, Structure, Visit,
    },
    Result,
};

#[derive(Clone, Copy)]
pub struct EmptyStructure;

impl Structure for EmptyStructure {
    type Activated = EmptyStructure;
    fn activate(self, _: &mut (), _: &BareContent) -> Self {
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
    fn update(self, _: ActivateUpdateArguments<(), L>) -> Result<bool> {
        Ok(true)
    }
}
