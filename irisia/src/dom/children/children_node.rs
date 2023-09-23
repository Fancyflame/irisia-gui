use crate::{
    dom::update::EMUpdateContent, structure::MapVisit, update_with::SpecificUpdate, UpdateWith,
};

use super::RenderMultiple;

pub trait ChildrenNodes
where
    Self: for<'a> Helper<'a, HelperModel = Self::Model>,
{
    type Model: RenderMultiple;
}

impl<T, M> ChildrenNodes for T
where
    T: for<'a> Helper<'a, HelperModel = M>,
    M: RenderMultiple,
{
    type Model = M;
}

pub trait Helper<'a>: Sized {
    type HelperModel: RenderMultiple;

    fn create_model(self, updater: EMUpdateContent<'a>) -> Self::HelperModel;
    fn update_model(
        self,
        model: &mut Self::HelperModel,
        updater: EMUpdateContent<'a>,
        equality_matters: &mut bool,
    );
}

impl<'a, T, M> Helper<'a> for T
where
    T: MapVisit<EMUpdateContent<'a>>,
    T::Output: SpecificUpdate<UpdateTo = M>,
    M: RenderMultiple + UpdateWith<T::Output> + 'static,
{
    type HelperModel = M;

    fn create_model(self, updater: EMUpdateContent<'a>) -> Self::HelperModel {
        M::create_with(self.map(&updater))
    }

    fn update_model(
        self,
        model: &mut Self::HelperModel,
        updater: EMUpdateContent<'a>,
        equality_matters: &mut bool,
    ) {
        *equality_matters &= model.update_with(self.map(&updater), *equality_matters);
    }
}
