use std::sync::Arc;

use crate::{
    application::content::GlobalContent,
    dom::{
        children::{ChildrenBox, ChildrenNodes},
        update::ApplyGlobalContent,
    },
    UpdateWith,
};

pub use peek_styles::PeekStyles;

mod peek_styles;

pub struct ChildrenSetter<'a> {
    set_children: &'a mut Option<ChildrenBox>,
    global_content: &'a Arc<GlobalContent>,
}

impl<'a> ChildrenSetter<'a> {
    pub(crate) fn new(
        set_children: &'a mut Option<ChildrenBox>,
        gc: &'a Arc<GlobalContent>,
    ) -> Self {
        Self {
            set_children,
            global_content: gc,
        }
    }

    pub fn set_children<T>(self, children: T) -> PeekStyles<'a, T::AliasUpdateTo>
    where
        T: ChildrenNodes<'a>,
    {
        let updater = children.map(&ApplyGlobalContent(self.global_content));
        let (cb, _) = ChildrenBox::update_option(self.set_children, updater, false);

        PeekStyles::new(
            cb.as_render_object()
                .as_any()
                .downcast_ref()
                .unwrap_or_else(|| unreachable!()),
        )
    }
}
