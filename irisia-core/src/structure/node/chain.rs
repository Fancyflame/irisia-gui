use crate::{
    application::event_comp::NewPointerEvent,
    element::SelfCache,
    structure::{
        activate::{ActivatedStructure, CacheUpdateArguments, Structure, UpdateCache, Visit},
        cache::NodeCache,
        layer::LayerRebuilder,
    },
    Result,
};

#[derive(Default)]
pub struct Chain<A, B>(pub(crate) A, pub(crate) B);

impl<A, B> Structure for Chain<A, B>
where
    A: Structure,
    B: Structure,
{
    type Activated = Chain<A::Activated, B::Activated>;

    fn activate(self, cache: &mut SelfCache<Self>) -> Self::Activated {
        Chain(self.0.activate(&mut cache.0), self.1.activate(&mut cache.1))
    }
}

impl<A, B> ActivatedStructure for Chain<A, B>
where
    A: ActivatedStructure,
    B: ActivatedStructure,
{
    type Cache = Chain<A::Cache, B::Cache>;

    fn element_count(&self) -> usize {
        self.0.element_count() + self.1.element_count()
    }
}

impl<A, B, L> UpdateCache<L> for Chain<A, B>
where
    A: UpdateCache<L>,
    B: UpdateCache<L>,
{
    fn update(self, args: CacheUpdateArguments<Self::Cache, L>) -> Result<bool> {
        let CacheUpdateArguments {
            offset,
            cache,
            global_content,
            layouter,
            equality_matters: mut unchange,
        } = args;
        let element_count = self.0.element_count();

        unchange &= self.0.update(CacheUpdateArguments {
            offset,
            cache: &mut cache.0,
            global_content: global_content.downgrade_lifetime(),
            layouter,
            equality_matters: unchange,
        })?;

        unchange &= self.1.update(CacheUpdateArguments {
            offset: offset + element_count,
            cache: &mut cache.1,
            global_content,
            layouter,
            equality_matters: unchange,
        })?;

        Ok(unchange)
    }
}

impl<A, B, V> Visit<V> for Chain<A, B>
where
    A: Visit<V>,
    B: Visit<V>,
{
    fn visit_at(&self, index: usize, visitor: &mut V) {
        self.0.visit_at(index, visitor);
        self.1.visit_at(index + self.element_count(), visitor);
    }
}

impl<A, B> NodeCache for Chain<A, B>
where
    A: NodeCache,
    B: NodeCache,
{
    fn render(&self, rebuilder: &mut LayerRebuilder) -> Result<()> {
        self.0.render(rebuilder)?;
        self.1.render(rebuilder)
    }

    fn emit_event(&mut self, new_event: &NewPointerEvent) -> bool {
        // cannot use `||` here, we need both two node to emit the event
        self.1.emit_event(new_event) | self.0.emit_event(new_event)
    }
}
