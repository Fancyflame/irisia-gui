use std::{marker::PhantomData, sync::Arc};

use crate::{
    application::{content::GlobalContent, event_comp::NodeEventMgr},
    element::{Element, UpdateOptions},
    structure::{slot::Slot, MapVisit, MapVisitor},
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
pub struct ApplyGlobalContent<'a>(pub(crate) &'a Arc<GlobalContent>);

impl<'a, El, Pr, Sty, Ch, Oc> MapVisitor<AddOne<El, Pr, Sty, Ch, Oc>> for ApplyGlobalContent<'a> {
    type Output = ElModelUpdate<'a, El, Pr, Sty, Ch, Oc>;
    fn map_visit(&self, data: AddOne<El, Pr, Sty, Ch, Oc>) -> Self::Output {
        ElModelUpdate {
            add_one: data,
            global_content: self.0,
        }
    }
}

impl<El, Pr, Sty, Ch, Cc, Oc> UpdateWith<ElModelUpdate<'_, El, Pr, Sty, Ch, Oc>>
    for ElementModel<El, Sty, Cc>
where
    El: Element + for<'sty> UpdateWith<UpdateOptions<'sty, Pr, Sty>>,
    Sty: StyleContainer + 'static,
    Ch: for<'a> ChildrenNodes<'a, AliasUpdateTo = Cc>,
    Cc: for<'a> UpdateWith<<Ch as MapVisit<ApplyGlobalContent<'a>>>::Output>,
    Oc: FnOnce(&Arc<ElementHandle<El>>),
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

        let element_handle = Arc::new(ElementHandle::new(
            global_content.clone(),
            UpdateOptions {
                props,
                styles: &styles,
            },
        ));
        on_create(&element_handle);

        ElementModel {
            independent_layer: None,
            slot_cache: Slot::new(children.map(&ApplyGlobalContent(global_content))),
            event_mgr: NodeEventMgr::new(),
            interact_region: None,
            expanded_children: None,
            draw_region: Default::default(),
            shared_part: element_handle,
            styles,
        }
    }

    fn update_with(
        &mut self,
        updater: ElModelUpdate<El, Pr, Sty, Ch, Oc>,
        mut equality_matters: bool,
    ) -> bool {
        let AddOne {
            props,
            styles,
            children,
            ..
        } = updater.add_one;

        equality_matters &= self.slot_cache.update_inner(
            children.map(&ApplyGlobalContent(self.shared_part.global())),
            equality_matters,
        );

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
