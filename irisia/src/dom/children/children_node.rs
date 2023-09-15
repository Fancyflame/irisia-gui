use crate::{
    dom::update::EMUpdateContent, structure::MapVisit, update_with::SpecificUpdate, UpdateWith,
};

use super::RenderMultiple;

pub trait ChildrenNodes: Sized {
    type Model: RenderMultiple;

    fn create_model<'a>(self, updater: EMUpdateContent) -> Self::Model;
    fn update_model<'a>(
        self,
        model: &mut Self::Model,
        updater: EMUpdateContent,
        equality_matters: &mut bool,
    );
}

type MapOutput<'a, T> = <T as MapVisit<EMUpdateContent<'a>>>::Output;

impl<T, M> ChildrenNodes for T
where
    T: for<'a> MapVisit<EMUpdateContent<'a>>,
    for<'a> MapOutput<'a, T>: SpecificUpdate<UpdateTo = M>,
    M: RenderMultiple + for<'a> UpdateWith<MapOutput<'a, T>> + 'static,
{
    type Model = M;

    fn create_model<'a>(self, updater: EMUpdateContent) -> Self::Model {
        M::create_with(self.map(&updater))
    }

    fn update_model<'a>(
        self,
        model: &mut Self::Model,
        updater: EMUpdateContent,
        equality_matters: &mut bool,
    ) {
        *equality_matters &= model.update_with(self.map(&updater), *equality_matters);
    }
}
