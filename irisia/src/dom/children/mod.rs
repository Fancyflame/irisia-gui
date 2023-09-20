pub use self::{children_node::ChildrenNodes, render_multiple::RenderMultiple};

mod children_node;
mod render_multiple;

pub(crate) struct ChildrenBox {
    structure: Box<dyn RenderMultiple>,
}

impl ChildrenBox {
    pub fn new<T>(value: T) -> Self
    where
        T: RenderMultiple,
    {
        ChildrenBox {
            structure: Box::new(value),
        }
    }

    pub fn as_render_multiple(&mut self) -> &mut dyn RenderMultiple {
        &mut *self.structure
    }
}
