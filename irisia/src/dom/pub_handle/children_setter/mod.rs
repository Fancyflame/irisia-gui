use std::{cell::RefMut, sync::Arc};

use crate::{
    application::{content::GlobalContent, redraw_scheduler::LayerId},
    dom::{
        children::{ChildrenBox, ChildrenNodes, RenderMultiple},
        layer::{SharedLayerCompositer, WeakLayerCompositer},
        EMUpdateContent,
    },
    primitive::Region,
    structure::{Visit, VisitMut},
    Result,
};

use self::visitors::{ApplyRegion, VisitStyles};

mod visitors;

pub struct LayoutElements<'a, T> {
    model: RefMut<'a, T>,
    layouted: bool,
}

impl<'a, T> LayoutElements<'a, T> {
    pub(crate) fn new<U>(
        children: U,
        children_box: RefMut<'a, Option<ChildrenBox>>,
        global_content: &'a Arc<GlobalContent>,
        children_layer: SharedLayerCompositer,
    ) -> Self
    where
        U: ChildrenNodes<Model = T>,
        T: RenderMultiple,
    {
        let updater = EMUpdateContent {
            global_content,
            parent_layer: children_layer,
        };

        let refmut = RefMut::map(children_box, |option| match option {
            Some(cb) => {
                let model=
                    cb.as_render_multiple()
                    .as_any()
                    .downcast_mut::<T>()
                    .expect("the type of children is not equal to previous's, these two is expected to be the same");

                children.update_model(model, updater, &mut false);
                model
            }
            place @ None => place
                .insert(ChildrenBox::new(children.create_model(updater)))
                .as_render_multiple()
                .as_any()
                .downcast_mut()
                .unwrap(),
        });

        Self {
            model: refmut,
            layouted: false,
        }
    }

    pub fn peek_styles<F, Sr>(&self, f: F)
    where
        F: FnMut(Sr),
        T: Visit<VisitStyles<F, Sr>>,
    {
        let _ = self.model.visit(&mut VisitStyles::new(f));
    }

    pub fn layout<F, Sr>(mut self, layouter: F) -> Result<()>
    where
        F: FnMut(Sr) -> Option<Region>,
        T: VisitMut<ApplyRegion<F, Sr>>,
    {
        self.model.visit_mut(&mut ApplyRegion::new(layouter))
    }
}
