use super::{BoxedModel, GeneralVModel};
use crate::model::{Model, UnitVModel, VModel, unit::UnitAssertion};

mod __sealed {
    /// 单元素虚模型断言
    pub trait UnitVModelAssertion {}
    impl<T: super::UnitVModel> UnitVModelAssertion for T {}
}

/// 通用单元素虚模型：与通用虚模型用法相同，但生成的模型带有单元素标记
pub trait GeneralUnitVModel: GeneralVModel + __sealed::UnitVModelAssertion {}

impl<T: UnitVModel> GeneralUnitVModel for T {}

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
