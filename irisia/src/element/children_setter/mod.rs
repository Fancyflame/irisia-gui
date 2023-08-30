use std::sync::Arc;

use crate::{
    application::{content::GlobalContent, redraw_scheduler::LayerId},
    dom::{
        children::{ChildrenBox, ChildrenNodes},
        EMUpdateContent,
    },
};

pub use layout_elements::LayoutElements;

mod layout_elements;

pub struct ChildrenSetter<'a> {
    set_children: &'a mut Option<ChildrenBox>,
    global_content: &'a Arc<GlobalContent>,
    dep_layer_id: LayerId,
}

impl<'a> ChildrenSetter<'a> {
    pub(crate) fn new(
        set_children: &'a mut Option<ChildrenBox>,
        gc: &'a Arc<GlobalContent>,
        dep_layer_id: LayerId,
    ) -> Self {
        Self {
            set_children,
            global_content: gc,
            dep_layer_id,
        }
    }

    pub fn set_children<T>(self, children: T) -> LayoutElements<'a, T::Model>
    where
        T: ChildrenNodes,
    {
        let updater = EMUpdateContent {
            global_content: self.global_content,
            dep_layer_id: self.dep_layer_id,
        };

        let model = match self.set_children {
            Some(cb) => {
                let model=cb.as_render_multiple()
                    .as_any()
                    .downcast_mut::<T::Model>()
                    .expect("the type of children is not equal to previous's, these two is expected to be the same");
                children.update_model(model, &updater, &mut false);
                model
            }
            None => {
                *self.set_children = Some(ChildrenBox::new(children.create_model(&updater)));
                self.set_children
                    .as_mut()
                    .unwrap()
                    .as_render_multiple()
                    .as_any()
                    .downcast_mut()
                    .unwrap()
            }
        };

        LayoutElements::new(model)
    }
}
