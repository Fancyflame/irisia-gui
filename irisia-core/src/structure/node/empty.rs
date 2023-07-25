use crate::{
    structure::activate::{
        ActivatedStructure, CacheUpdateArguments, Structure, UpdateCache, Visit,
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
