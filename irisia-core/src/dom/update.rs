use std::{marker::PhantomData, sync::Arc};

use crate::{
    application::{content::GlobalContent, event_comp::NodeEventMgr},
    element::{update_element::UpdateOptions, Element, UpdateElement},
    structure::MapVisitor,
    style::StyleContainer,
    update_with::SpecificUpdate,
    UpdateWith,
};

use super::{
    children::{ChildrenBox, RenderObject, SetChildren},
    ElementModel,
};

pub struct AddOne<El, Pr, Sty, Ch, Oc> {
    _el: PhantomData<El>,
    pub(super) props: Pr,
    pub(super) styles: Sty,
    pub(super) children: Ch,
    pub(super) on_create: Oc,
}

pub fn add_one<El, Pr, Sty, Ch, Oc>(
    props: Pr,
    styles: Sty,
    children: Ch,
    on_create: Oc,
) -> AddOne<El, Pr, Sty, Ch, Oc> {
    AddOne {
        _el: Default::default(),
        props,
        styles,
        children,
        on_create,
    }
}

pub struct ApplyGlobalContent<'a>(pub(crate) &'a Arc<GlobalContent>);

impl<'a, El, Pr, Sty, Ch, Oc> MapVisitor<AddOne<El, Pr, Sty, Ch, Oc>> for ApplyGlobalContent<'a> {
    type Output = (AddOne<El, Pr, Sty, Ch, Oc>, &'a Arc<GlobalContent>);
    fn map_visit(&self, data: AddOne<El, Pr, Sty, Ch, Oc>) -> Self::Output {
        (data, self.0)
    }
}

impl<El, Pr, Sty, Ch, Oc> UpdateWith<(AddOne<El, Pr, Sty, Ch, Oc>, &Arc<GlobalContent>)>
    for ElementModel<El, Sty>
where
    El: Element + for<'a> UpdateWith<UpdateOptions<'a, Pr, Sty, Ch>>,
    Sty: StyleContainer + 'static,
    Ch: RenderObject,
    Oc: FnOnce(&GlobalContent),
{
    fn create_with(updater: (AddOne<El, Pr, Sty, Ch, Oc>, &Arc<GlobalContent>)) -> Self {
        let (
            AddOne {
                _el,
                props,
                styles,
                children,
                on_create,
            },
            global_content,
        ) = updater;

        let mut children_box = None;
        let mut computed_size = Default::default();
        let update_options = UpdateOptions {
            props,
            styles: &styles,
            children,
            updater: UpdateElement::new(
                SetChildren::Create(&mut children_box),
                global_content,
                &mut computed_size,
            ),
        };

        let element = El::create_with(update_options);
        on_create(&global_content);

        ElementModel {
            element,
            styles,
            independent_layer: None,
            global_content: global_content.clone(),
            event_mgr: NodeEventMgr::new(),
            interact_region: None,
            computed_size,
            expanded_children: children_box.unwrap_or_else(|| ChildrenBox::create_with(())),
        }
    }

    fn update_with(
        &mut self,
        updater: (AddOne<El, Pr, Sty, Ch, Oc>, &Arc<GlobalContent>),
        equality_matters: bool,
    ) -> bool {
        let AddOne {
            props,
            styles,
            children,
            ..
        } = updater.0;

        self.computed_size = Default::default();

        let update_options = UpdateOptions {
            props,
            styles: &styles,
            children,
            updater: UpdateElement::new(
                SetChildren::Update(&mut self.expanded_children),
                &self.global_content,
                &mut self.computed_size,
            ),
        };

        self.element.update_with(update_options, equality_matters) && equality_matters
    }
}

impl<El, Pr, Sty, Ch, Oc> SpecificUpdate for (AddOne<El, Pr, Sty, Ch, Oc>, &Arc<GlobalContent>) {
    type UpdateTo = ElementModel<El, Sty>;
}
