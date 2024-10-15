use super::{StructureCreate, VisitBy};
use crate::{
    el_model::{EMCreateCtx, ElementAccess, ElementModel},
    element::FromUserProps,
    ElementInterfaces,
};

pub fn single<'a, El, Cp>(
    props: <El::Props<'a> as FromUserProps>::Props,
    child_props: Cp,
    slot: impl StructureCreate + 'a,
    on_create: impl FnOnce(&ElementAccess) + 'a,
) -> impl StructureCreate<Target = ElementModel<El, Cp>> + 'a
where
    Cp: 'static,
    El: ElementInterfaces,
{
    move |context: &EMCreateCtx| {
        let props = <El::Props<'a> as FromUserProps>::take(props);
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
