use super::{BoxedModel, GeneralVModel};
use crate::model::{Model, VModel, unit::UnitAssertion};

pub trait GeneralUnitVModel: GeneralVModel + UnitAssertion {}

impl VModel for dyn GeneralUnitVModel {
    type Storage = BoxedUnitModel;
    fn create(&self, ctx: &crate::model::ModelCreateCtx) -> Self::Storage {
        BoxedUnitModel(self.dyn_create(ctx))
    }
    fn update(&self, storage: &mut Self::Storage, ctx: &crate::model::ModelCreateCtx) {
        self.dyn_update(&mut storage.0, ctx);
    }
}

pub struct BoxedUnitModel(BoxedModel);

impl Model for BoxedUnitModel {
    fn visit_raw(&self, f: crate::model::VisitModelFn) {
        self.0.visit_raw(f);
    }
}

impl UnitAssertion for BoxedUnitModel {}
