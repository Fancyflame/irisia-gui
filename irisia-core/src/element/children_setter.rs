use std::sync::Arc;

use crate::{
    application::content::GlobalContent,
    dom::{
        children::{ChildrenBox, ChildrenNodes},
        update::ApplyGlobalContent,
    },
    UpdateWith,
};

use super::style_peeker::PeekStyles;

pub struct ChildrenSetter<'a> {
    set_children: &'a mut Option<ChildrenBox>,
    global_content: ApplyGlobalContent<'a>,
    changed: &'a mut bool,
}

impl<'a> ChildrenSetter<'a> {
    pub(crate) fn new(
        set_children: &'a mut Option<ChildrenBox>,
        gc: &'a Arc<GlobalContent>,
        equality_matters: &'a mut bool,
    ) -> Self {
        Self {
            set_children,
            global_content: ApplyGlobalContent { global_content: gc },
            changed: equality_matters,
        }
    }

    pub fn set_children<'b, T>(&'b mut self, children: T) -> PeekStyles<'b, T::AliasUpdateTo>
    where
        T: ChildrenNodes<'a>,
    {
        let updater = children.map(&self.global_content);
        let (cb, changed) = ChildrenBox::update_option(self.set_children, updater, *self.changed);
        *self.changed &= changed;

        PeekStyles::new(
            cb.as_render_object()
                .as_any()
                .downcast_ref()
                .unwrap_or_else(|| unreachable!()),
        )
    }
}
