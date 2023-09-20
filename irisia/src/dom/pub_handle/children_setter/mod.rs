use std::{cell::RefMut, sync::Arc};

use crate::{
    application::content::GlobalContent,
    dom::{
        children::{ChildrenBox, ChildrenNodes, RenderMultiple},
        layer::SharedLayerCompositer,
        EMUpdateContent,
    },
    primitive::Region,
    style::StyleContainer,
    Result, StyleReader,
};

mod visitors;

pub struct LayoutElements<'a, T> {
    model: RefMut<'a, T>,
    layouted: bool,
}

impl<'a, T> LayoutElements<'a, T>
where
    T: RenderMultiple,
{
    pub(crate) fn new<U>(
        children: U,
        children_box: RefMut<'a, Option<ChildrenBox>>,
        global_content: &'a Arc<GlobalContent>,
        children_layer: SharedLayerCompositer,
    ) -> Self
    where
        U: ChildrenNodes<Model = T>,
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

    pub fn peek_styles<F, Sr>(&self, mut f: F)
    where
        F: FnMut(Sr),
        Sr: StyleReader,
    {
        let _ = self
            .model
            .peek_styles(&mut |inside_style_box| f(inside_style_box.read()));
    }

    pub fn layout<F, Sr>(mut self, mut layouter: F) -> Result<()>
    where
        F: FnMut(Sr) -> Option<Region>,
        Sr: StyleReader,
    {
        self.model
            .layout(&mut |inside_style_box| layouter(inside_style_box.read()))
    }
}
