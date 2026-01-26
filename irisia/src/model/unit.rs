use crate::{
    model::{Model, VModel},
    prim_element::Element,
};

pub(super) use __sealed::UnitAssertion;

mod __sealed {
    /// 单元素模型断言标记
    pub trait UnitAssertion {}
}

pub trait UnitVModel
where
    Self: VModel<Storage: UnitModel>,
{
}
impl<T: VModel<Storage: UnitModel> + ?Sized> UnitVModel for T {}

pub trait UnitModel: Model + UnitAssertion {
    fn visit_as_unit<F, Data, R>(&self, mut f: F) -> R
    where
        F: FnOnce(Element, Option<&Data>) -> R,
        Data: 'static,
    {
        let mut result = None;

        self.0.visit_raw(&mut |el, any| {
            result = Some(f(el, any));
        });

        result
            .unwrap_or_else(|| unreachable!("unit model assertion failed, this is an internal bug"))
    }
}
impl<T: Model + UnitAssertion + ?Sized> UnitModel for T {}
