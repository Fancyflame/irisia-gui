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
    set_children: Option<&'a mut Option<ChildrenBox>>,
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
            set_children: Some(set_children),
            global_content: gc,
            dep_layer_id,
        }
    }

    pub fn set_children<T>(mut self, children: T) -> LayoutElements<'a, T::Model>
    where
        T: ChildrenNodes,
    {
        let updater = EMUpdateContent {
            global_content: self.global_content,
            dep_layer_id: self.dep_layer_id,
        };

        let model = match self.set_children.take().unwrap() {
            Some(cb) => {
                let model=cb.as_render_multiple()
                    .as_any()
                    .downcast_mut::<T::Model>()
                    .expect("the type of children is not equal to previous's, these two is expected to be the same");
                children.update_model(model, &updater, &mut false);
                model
            }
            place @ None => {
                *place = Some(ChildrenBox::new(children.create_model(&updater)));
                place
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

impl Drop for ChildrenSetter<'_> {
    fn drop(&mut self) {
        // if the children box was not setted, then initialize
        // with a children box with empty node.
        if let Some(cb @ None) = &mut self.set_children {
            **cb = Some(ChildrenBox::new(()));
        }
    }
}
