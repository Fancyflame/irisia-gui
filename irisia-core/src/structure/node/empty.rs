use crate::{
    application::event_comp::NewPointerEvent,
    structure::{
        activate::{ActivatedStructure, CacheUpdateArguments, Structure, UpdateCache, Visit},
        cache::NodeCache,
        layer::LayerRebuilder,
    },
    Result,
};

impl Structure for () {
    type Activated = ();
    fn activate(self, _: &mut ()) -> () {
        ()
    }
}

impl ActivatedStructure for () {
    type Cache = ();
    fn element_count(&self) -> usize {
        0
    }
}

impl<V> Visit<V> for () {
    fn visit_at(&self, _: usize, _: &mut V) {}
}

impl<L> UpdateCache<L> for () {
    fn update(self, _: CacheUpdateArguments<(), L>) -> Result<bool> {
        Ok(true)
    }
}

impl NodeCache for () {
    fn render(&self, _: &mut LayerRebuilder) -> Result<()> {
        Ok(())
    }

    fn emit_event(&mut self, _: &NewPointerEvent) -> bool {
        false
    }
}
