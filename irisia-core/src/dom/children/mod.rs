use crate::{update_with::SpecificUpdate, UpdateWith};

pub(crate) use self::render_object::RenderObject;

mod render_object;

pub(crate) struct ChildrenBox {
    structure: Box<dyn RenderObject>,
}

impl ChildrenBox {
    pub fn as_render_object(&mut self) -> &mut dyn RenderObject {
        &mut *self.structure
    }
}

impl<T> UpdateWith<T> for ChildrenBox
where
    T: SpecificUpdate,
    T::UpdateTo: UpdateWith<T> + RenderObject + 'static,
{
    fn create_with(updater: T) -> Self {
        ChildrenBox {
            structure: Box::new(T::UpdateTo::create_with(updater)) as _,
        }
    }

    fn update_with(&mut self, updater: T, equality_matters: bool) -> bool {
        let place: &mut T::UpdateTo = self
            .structure
            .as_any()
            .downcast_mut()
            .expect("children structure cannot changed");
        place.update_with(updater, equality_matters) && equality_matters
    }
}

pub(crate) enum SetChildren<'a> {
    Create(&'a mut Option<ChildrenBox>),
    Update(&'a mut ChildrenBox),
}
