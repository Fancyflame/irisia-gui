use crate::{
    structure::visit::{ActivatedStructure, BareContentWrapper, Renderable, Structure, Visit},
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
        content: &BareContentWrapper,
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
    fn render_at(
        self,
        index: usize,
        cache: &mut Self::Cache,
        mut bare_content: BareContentWrapper,
        layouter: &mut L,
    ) -> Result<()> {
        let element_count = self.0.element_count();
        self.0.render_at(
            index,
            &mut cache.0,
            BareContentWrapper(bare_content.0.downgrade_lifetime()),
            layouter,
        )?;
        self.1
            .render_at(index + element_count, &mut cache.1, bare_content, layouter)
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
