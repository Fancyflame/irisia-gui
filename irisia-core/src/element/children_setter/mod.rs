use std::sync::Arc;

use crate::{
    application::{content::GlobalContent, redraw_scheduler::LayerId},
    dom::{
        children::{ChildrenBox, ChildrenNodes},
        EMUpdateContent,
    },
    UpdateWith,
};

pub use peek_styles::PeekStyles;

mod peek_styles;

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

    pub fn set_children<T>(self, children: T) -> PeekStyles<'a, T::AliasUpdateTo>
    where
        T: ChildrenNodes<'a>,
    {
        let updater = children.map(&EMUpdateContent {
            global_content: self.global_content,
            dep_layer_id: self.dep_layer_id,
        });
        let (cb, _) = ChildrenBox::update_option(self.set_children, updater, false);

        PeekStyles::new(
            cb.as_render_multiple()
                .as_any()
                .downcast_ref()
                .unwrap_or_else(|| unreachable!()),
        )
    }
}
