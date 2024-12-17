use crate::{
    el_model::{EMCreateCtx, ElementAccess, ElementModel},
    ElementInterfaces,
};

use super::{
    iter::{ModelMapper, ModelMapperImplements, VisitModel},
    DesiredVModel, VModel,
};

pub struct Unit<El, Cp, Slt, Oc>
where
    El: ElementInterfaces,
    Oc: Fn(&ElementAccess) + 'static,
{
    pub props: El::Props,
    pub child_data: Cp,
    pub slot: Slt,
    pub on_create: Oc,
}

impl<El, Cp, Slt, Oc> VModel for Unit<El, Cp, Slt, Oc>
where
    El: ElementInterfaces,
    Slt: DesiredVModel<El::ChildMapper> + Clone,
    Cp: Clone + 'static,
    Oc: Fn(&ElementAccess),
{
    type Storage = ElementModel<El, Cp, Slt>;
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        let em = ElementModel::new(ctx, &self.props, self.child_data.clone(), self.slot.clone());
        (self.on_create)(em.access());
        em
    }
    fn update(&self, storage: &mut Self::Storage, _: &EMCreateCtx) {
        *storage.slot.write() = self.slot.clone();
    }
}

impl<M, El, Cp, Slt> VisitModel<M> for ElementModel<El, Cp, Slt>
where
    M: ModelMapper + ModelMapperImplements<El, Cp, Slt>,
{
    fn visit(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapRef<'_>)) {
        f(M::map_ref(self))
    }
    fn visit_mut(&mut self, f: &mut dyn FnMut(<M as ModelMapper>::MapMut<'_>)) {
        f(M::map_mut(self))
    }
}

impl<El, Cp, Slt, Oc> Clone for Unit<El, Cp, Slt, Oc>
where
    El: ElementInterfaces,
    El::Props: Clone,
    Cp: Clone,
    Slt: Clone,
    Oc: Fn(&ElementAccess) + Clone,
{
    fn clone(&self) -> Self {
        Self {
            props: self.props.clone(),
            child_data: self.child_data.clone(),
            slot: self.slot.clone(),
            on_create: self.on_create.clone(),
        }
    }
}
