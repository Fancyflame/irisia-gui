use std::{
    cell::RefMut,
    rc::{Rc, Weak},
};

use crate::{
    application::{content::GlobalContent, redraw_scheduler::RedrawObject},
    dom::{
        children::{ChildrenBox, ChildrenNodes, RenderMultiple},
        EMUpdateContent,
    },
    primitive::Region,
    style::StyleContainer,
    Result, StyleReader,
};

pub struct LayoutElements<'a, T>(RefMut<'a, T>);

impl<'a, T> LayoutElements<'a, T>
where
    T: RenderMultiple,
{
    pub(crate) fn new<U>(
        children: U,
        children_box: RefMut<'a, Option<ChildrenBox>>,
        global_content: &'a Rc<GlobalContent>,
        children_layer: Weak<dyn RedrawObject>,
    ) -> Self
    where
        U: ChildrenNodes<Model = T>,
    {
        let updater = EMUpdateContent {
            global_content,
            parent_layer: Some(children_layer),
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

        Self(refmut)
    }

    pub fn peek_styles<F, Sr>(&self, mut f: F)
    where
        F: FnMut(Sr),
        Sr: StyleReader,
    {
        let _ = self
            .0
            .peek_styles(&mut |inside_style_box| f(inside_style_box.read()));
    }

    pub fn layout<F, Sr>(self, mut layouter: F) -> Result<()>
    where
        F: FnMut(Sr) -> Option<Region>,
        Sr: StyleReader,
    {
        self.0
            .layout(&mut |inside_style_box| layouter(inside_style_box.read()))
    }
}
