use std::{
    marker::PhantomData,
    sync::{atomic::AtomicBool, Arc, Mutex as StdMutex},
};

use tokio::sync::RwLock;

use crate::{
    application::{content::GlobalContent, event_comp::NodeEventMgr, redraw_scheduler::LayerId},
    element::{Element, UpdateOptions},
    event::EventDispatcher,
    structure::{slot::Slot, MapVisit, MapVisitor},
    style::StyleContainer,
    update_with::SpecificUpdate,
    UpdateWith,
};

use super::{
    children::ChildrenNodes,
    data_structure::{maybe_shared::MaybeShared, ElementHandle, LayerSharedPart},
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

impl<El, Pr, Sty, Ch, Sc, Oc> UpdateWith<ElementModelUpdater<'_, El, Pr, Sty, Ch, Oc>>
    for ElementModel<El, Sty, Sc>
where
    El: Element + for<'sty> UpdateWith<UpdateOptions<'sty, El, Pr, Sty>>,
    Sty: StyleContainer + 'static,
    Ch: for<'a> ChildrenNodes<'a, AliasUpdateTo = Sc>,
    Sc: for<'a> UpdateWith<<Ch as MapVisit<EMUpdateContent<'a>>>::Output>,
    Oc: FnOnce(&Arc<ElementHandle<El>>),
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
            let eh = Arc::new(ElementHandle {
                el: RwLock::new(None),
                ed: EventDispatcher::new(),
                global_content: global_content.clone(),
                lock_independent_layer: AtomicBool::new(false),
                dep_layer_id: StdMutex::new(dep_layer_id),
            });

            // hold the lock prevent from being accessed
            let mut write = eh.el.blocking_write();

            let el = El::create_with(UpdateOptions {
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
            parent_layer_id: dep_layer_id,
            pub_shared: element_handle.clone(),
            expanded_children: None,
            draw_region: Default::default(),
            interact_region: None,
        });

        ElementModel {
            event_mgr: NodeEventMgr::new(),
            slot_cache: Slot::new(children.map(&EMUpdateContent {
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

        self.shared.borrow_mut().parent_layer_id = dep_layer_id;

        // if self is unique, then self has no indep layer. we need to update.
        if self.shared.is_unique() {
            *self.pub_shared.dep_layer_id.lock().unwrap() = dep_layer_id;
        }

        equality_matters &= self.slot_cache.update_inner(
            children.map(&EMUpdateContent {
                global_content: self.pub_shared.global(),
                dep_layer_id,
            }),
            equality_matters,
        );

        equality_matters
            & self.pub_shared.el_write_clean().update_with(
                UpdateOptions {
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
    Ch: ChildrenNodes<'a>,
{
    type UpdateTo = ElementModel<El, Sty, Ch::AliasUpdateTo>;
}
