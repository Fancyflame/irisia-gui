use std::{marker::PhantomData, sync::Arc};

use crate::{
    application::{content::GlobalContent, event_comp::NodeEventMgr},
    element::{Element, UpdateOptions},
    structure::MapVisitor,
    style::StyleContainer,
    update_with::SpecificUpdate,
    UpdateWith,
};

use super::{children::ChildrenNodes, shared::ElementHandle, ElementModel};

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

#[doc(hidden)]
pub struct ElModelUpdate<'a, El, Pr, Sty, Ch, Oc> {
    pub(crate) add_one: AddOne<El, Pr, Sty, Ch, Oc>,
    pub(crate) global_content: &'a Arc<GlobalContent>,
}

#[derive(Clone, Copy)]
pub struct ApplyGlobalContent<'a> {
    pub(crate) global_content: &'a Arc<GlobalContent>,
}

impl<'a, El, Pr, Sty, Ch, Oc> MapVisitor<AddOne<El, Pr, Sty, Ch, Oc>> for ApplyGlobalContent<'a> {
    type Output = ElModelUpdate<'a, El, Pr, Sty, Ch, Oc>;
    fn map_visit(&self, data: AddOne<El, Pr, Sty, Ch, Oc>) -> Self::Output {
        ElModelUpdate {
            add_one: data,
            global_content: self.global_content,
        }
    }
}

impl<'a, El, Pr, Sty, Ch, Oc> UpdateWith<ElModelUpdate<'a, El, Pr, Sty, Ch, Oc>>
    for ElementModel<El, Sty, <Ch as ChildrenNodes<'a>>::AliasUpdateTo>
where
    El: Element + for<'sty> UpdateWith<UpdateOptions<'sty, Pr, Sty>>,
    Sty: StyleContainer + 'static,
    Ch: ChildrenNodes<'a>,
    Oc: FnOnce(&GlobalContent),
{
    fn create_with(updater: ElModelUpdate<El, Pr, Sty, Ch, Oc>) -> Self {
        let ElModelUpdate {
            add_one:
                AddOne {
                    _el,
                    props,
                    styles,
                    children,
                    on_create,
                },
            global_content,
        } = updater;

        on_create(&global_content);

        ElementModel {
            _slot_type: PhantomData,
            independent_layer: None,
            event_mgr: NodeEventMgr::new(),
            interact_region: None,
            expanded_children: None,
            draw_region: Default::default(),
            shared_part: Arc::new(ElementHandle::new(
                global_content.clone(),
                UpdateOptions {
                    props,
                    styles: &styles,
                },
            )),
            styles,
        }
    }

    fn update_with(
        &mut self,
        updater: ElModelUpdate<El, Pr, Sty, Ch, Oc>,
        equality_matters: bool,
    ) -> bool {
        let AddOne {
            props,
            styles,
            children,
            ..
        } = updater.add_one;

        let mut el = self.shared_part.el_mut();

        let update_options = UpdateOptions {
            props,
            styles: &styles,
        };

        equality_matters & el.update_with(update_options, equality_matters)
    }
}

impl<'a, El, Pr, Sty, Ch, Oc> SpecificUpdate for ElModelUpdate<'a, El, Pr, Sty, Ch, Oc>
where
    El: Element,
    Ch: ChildrenNodes<'a>,
{
    type UpdateTo = ElementModel<El, Sty, Ch::AliasUpdateTo>;
}
