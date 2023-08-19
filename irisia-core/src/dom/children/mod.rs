use crate::{update_with::SpecificUpdate, UpdateWith};

pub(crate) use self::{render_multiple::RenderMultiple, trait_alias::ChildrenNodes};

mod render_multiple;
pub(crate) mod trait_alias;

pub(crate) struct ChildrenBox {
    structure: Box<dyn RenderMultiple>,
}

impl ChildrenBox {
    pub fn as_render_multiple(&mut self) -> &mut dyn RenderMultiple {
        &mut *self.structure
    }
}

impl<T> UpdateWith<T> for ChildrenBox
where
    T: SpecificUpdate,
    T::UpdateTo: UpdateWith<T> + RenderMultiple + 'static,
{
    fn create_with(updater: T) -> Self {
        ChildrenBox {
            structure: Box::new(T::UpdateTo::create_with(updater)) as _,
        }
    }

    fn update_with(&mut self, updater: T, equality_matters: bool) -> bool {
        let place: &mut T::UpdateTo = self.structure.as_any().downcast_mut().expect(
            "the type of children is not equal to previous's, these two is expected to be the same",
        );
        place.update_with(updater, equality_matters) && equality_matters
    }
}
