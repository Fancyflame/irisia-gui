use super::{StructureCreate, VisitBy};
use crate::{
    el_model::{EMCreateCtx, ElementAccess, ElementModel},
    ElementInterfaces,
};

pub fn single<'a, El, Cp>(
    props: El::Props<'a>,
    child_props: Cp,
    slot: impl StructureCreate<El::SlotData> + 'a,
    on_create: impl FnOnce(&ElementAccess) + 'a,
) -> impl StructureCreate<Cp, Target = ElementModel<El, Cp>> + 'a
where
    Cp: 'static,
    El: ElementInterfaces,
{
    move |context: &EMCreateCtx| {
        let em = ElementModel::new(context, props, child_props, slot);
        on_create(em.access());
        em
    }
}

impl<El, Cp> VisitBy<Cp> for ElementModel<El, Cp>
where
    Cp: 'static,
    El: ElementInterfaces,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor<Cp>,
    {
        v.visit(self)
    }

    fn visit_mut<V>(&mut self, v: &mut V) -> crate::Result<()>
    where
        V: super::VisitorMut<Cp>,
    {
        v.visit_mut(self)
    }
}
