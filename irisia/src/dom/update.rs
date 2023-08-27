use std::{
    marker::PhantomData,
    sync::{Arc, RwLock as StdRwLock},
};

use tokio::sync::RwLock;

use crate::{
    application::{content::GlobalContent, event_comp::NodeEventMgr, redraw_scheduler::LayerId},
    element::{Element, UpdateElement},
    event::EventDispatcher,
    structure::{slot::Slot, MapVisitor},
    style::StyleContainer,
    update_with::SpecificUpdate,
    UpdateWith,
};

use super::{
    children::ChildrenNodes,
    data_structure::{
        maybe_shared::MaybeShared, ElementHandle, FullElementHandle, LayerInfo, LayerSharedPart,
    },
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

#[derive(Clone, Copy)]
pub struct EMUpdateContent<'a> {
    pub(crate) global_content: &'a Arc<GlobalContent>,
    pub(crate) dep_layer_id: LayerId,
}

impl<'a, El, Pr, Sty, Ch, Oc> MapVisitor<AddOne<El, Pr, Sty, Ch, Oc>> for EMUpdateContent<'a> {
    type Output = ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc>;
    fn map_visit(&self, data: AddOne<El, Pr, Sty, Ch, Oc>) -> Self::Output {
        ElementModelUpdater {
            add_one: data,
            content: *self,
        }
    }
}

#[doc(hidden)]
pub struct ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc> {
    pub(crate) add_one: AddOne<El, Pr, Sty, Ch, Oc>,
    pub(crate) content: EMUpdateContent<'a>,
}

impl<El, Pr, Sty, Ch, Oc> UpdateWith<ElementModelUpdater<'_, El, Pr, Sty, Ch, Oc>>
    for ElementModel<El, Sty, Ch::Model>
where
    El: Element + for<'sty> UpdateWith<UpdateElement<'sty, El, Pr, Sty>>,
    Sty: StyleContainer + 'static,
    Ch: ChildrenNodes,
    Oc: FnOnce(&FullElementHandle<El>),
{
    fn create_with(updater: ElementModelUpdater<El, Pr, Sty, Ch, Oc>) -> Self {
        let ElementModelUpdater {
            add_one:
                AddOne {
                    _el,
                    props,
                    styles,
                    children,
                    on_create,
                },
            content:
                EMUpdateContent {
                    global_content,
                    dep_layer_id,
                },
        } = updater;

        let element_handle = {
            let eh = FullElementHandle {
                el: Arc::new(RwLock::new(None)),
                base: Arc::new(ElementHandle {
                    ed: EventDispatcher::new(),
                    global_content: global_content.clone(),
                    layer_info: StdRwLock::new(LayerInfo {
                        acquire_independent_layer: false,
                        parent_layer_id: dep_layer_id,
                        indep_layer_id: None,
                    }),
                }),
            };

            // hold the lock prevent from being accessed
            let mut write = eh.el.blocking_write();

            let el = El::create_with(UpdateElement {
                props,
                styles: &styles,
                handle: &eh,
            });

            *write = Some(el);
            drop(write);
            eh
        };

        on_create(&element_handle);

        let shared = MaybeShared::new(LayerSharedPart {
            pub_shared: element_handle.clone(),
            expanded_children: None,
            draw_region: Default::default(),
            interact_region: None,
        });

        ElementModel {
            event_mgr: NodeEventMgr::new(),
            slot_cache: Slot::new(children.create_model(&EMUpdateContent {
                global_content,
                dep_layer_id,
            })),
            styles,
            shared,
            pub_shared: element_handle,
        }
    }

    fn update_with(
        &mut self,
        updater: ElementModelUpdater<El, Pr, Sty, Ch, Oc>,
        mut equality_matters: bool,
    ) -> bool {
        let ElementModelUpdater {
            add_one:
                AddOne {
                    _el,
                    props,
                    styles,
                    children,
                    on_create: _,
                },
            content:
                EMUpdateContent {
                    global_content: _,
                    dep_layer_id,
                },
        } = updater;

        self.pub_shared.layer_info.write().unwrap().parent_layer_id = dep_layer_id;

        children.update_model(
            &mut self.slot_cache.borrow_mut(),
            &EMUpdateContent {
                global_content: self.pub_shared.global(),
                dep_layer_id,
            },
            &mut equality_matters,
        );

        equality_matters
            & self.pub_shared.el_write_clean().update_with(
                UpdateElement {
                    handle: &self.pub_shared,
                    props,
                    styles: &styles,
                },
                equality_matters,
            )
    }
}

impl<'a, El, Pr, Sty, Ch, Oc> SpecificUpdate for ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc>
where
    El: Element,
    Ch: ChildrenNodes,
{
    type UpdateTo = ElementModel<El, Sty, Ch::Model>;
}
