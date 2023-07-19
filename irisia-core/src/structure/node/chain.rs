use crate::{
    element::render_content::BareContent,
    structure::activate::{
        ActivateUpdateArguments, ActivatedStructure, Renderable, Structure, Visit,
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

    fn activate(
        self,
        cache: &mut <Self::Activated as ActivatedStructure>::Cache,
        content: &BareContent,
    ) -> Self::Activated {
        Chain(
            self.0.activate(&mut cache.0, content),
            self.1.activate(&mut cache.1, content),
        )
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

impl<A, B, L> Renderable<L> for Chain<A, B>
where
    A: Renderable<L>,
    B: Renderable<L>,
{
    fn update(self, args: ActivateUpdateArguments<Self::Cache, L>) -> Result<bool> {
        let ActivateUpdateArguments {
            offset,
            cache,
            mut bare_content,
            layouter,
            equality_matters: mut everything_the_same,
        } = args;
        let element_count = self.0.element_count();

        everything_the_same &= self.0.update(ActivateUpdateArguments {
            offset,
            cache: &mut cache.0,
            bare_content: bare_content.downgrade_lifetime(),
            layouter,
            equality_matters: everything_the_same,
        })?;

        everything_the_same &= self.1.update(ActivateUpdateArguments {
            offset: offset + element_count,
            cache: &mut cache.1,
            bare_content,
            layouter,
            equality_matters: everything_the_same,
        })?;

        Ok(everything_the_same)
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
