use super::{StructureCreate, VisitBy};
use crate::{
    el_model::{EMCreateCtx, ElementAccess, ElementModel, SharedEM},
    style::ReadStyle,
    ElementInterfaces,
};

pub fn single<'a, El>(
    props: El::Props<'a>,
    styles: impl ReadStyle + 'static,
    slot: impl StructureCreate + 'a,
    on_create: impl FnOnce(ElementAccess) + 'a,
) -> impl StructureCreate<Target = SharedEM<El>> + 'a
where
    El: ElementInterfaces,
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
}
