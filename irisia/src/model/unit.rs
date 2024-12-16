use crate::{
    el_model::{EMCreateCtx, ElementModel},
    hook::ProviderObject,
    ElementInterfaces,
};

use super::{DesiredVModel, VModel};

pub struct Unit<El, Cp, Slt>
where
    El: ElementInterfaces,
{
    pub props: El::Props,
    pub child_data: Cp,
    pub slot: ProviderObject<Slt>,
}

impl<El, Cp, Slt> VModel for Unit<El, Cp, Slt>
where
    El: ElementInterfaces,
    Slt: DesiredVModel<El::AcceptChild>,
    Cp: Clone + 'static,
{
    type Storage = ElementModel<El, Cp>;
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        ElementModel::new(ctx, &self.props, self.child_data.clone(), self.slot.clone())
    }
    fn update(&self, _: &mut Self::Storage, _: &EMCreateCtx) {
        // never update
    }
}
