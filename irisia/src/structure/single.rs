use super::{StructureCreate, VisitBy};
use crate::{
    el_model::{EMCreateCtx, ElementAccess, ElementModel, SharedEM},
    style::ReadStyle,
    ElementInterfaces,
};

pub fn single<'a, El, Sty>(
    props: El::Props<'a>,
    styles: Sty,
    slot: impl StructureCreate + 'a,
    on_create: impl Fn(ElementAccess) + 'a,
) -> impl StructureCreate<Target = SharedEM<El>> + 'a
where
    El: ElementInterfaces,
    Sty: ReadStyle + 'static,
{
    move |context: &EMCreateCtx| {
        let em = ElementModel::new(context, props, styles, slot);
        on_create(em.access());
        em
    }
}

impl<El> VisitBy for SharedEM<El>
where
    El: ElementInterfaces,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor,
    {
        v.visit(self)
    }

    fn len(&self) -> usize {
        1
    }
}
