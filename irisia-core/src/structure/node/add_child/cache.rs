use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    element::{render_content::BareContent, ElementHandle, InitContent},
    event::{event_dispatcher::EventDispatcher, standard::ElementAbandoned},
    structure::{cache_box::CacheBox, layer::SharedLayerCompositer, Element},
};

pub struct AddChildCache<El> {
    pub(super) element: Arc<Mutex<El>>,
    pub(super) init_content: InitContent<El>,
    pub(super) cache_box: CacheBox,
    pub(super) independent_layer: Option<SharedLayerCompositer>,
}

impl<El> AddChildCache<El> {
    pub(super) fn new<F, Oc>(content: &BareContent, creator: F, on_create: Oc) -> Self
    where
        F: FnOnce(&InitContent<El>) -> El,
        El: Element,
        Oc: FnOnce(&InitContent<El>),
    {
        let event_dispatcher = EventDispatcher::new();
        let element_handle = ElementHandle::new(event_dispatcher.clone(), content.focusing.clone());

        let mut init_content = None;
        let element = Arc::new_cyclic(|weak| {
            let ic = InitContent {
                _prevent_user_init: (),
                app: weak.clone(),
                window_event_dispatcher: content.window_event_dispatcher.clone(),
                window: content.window.clone(),
                element_handle,
                close_handle: content.close_handle,
            };

            let el = creator(&ic);
            init_content = Some(ic);
            Mutex::new(el)
        });
        let init_content = init_content.unwrap();
        on_create(&init_content);

        AddChildCache {
            element,
            init_content,
            cache_box: CacheBox::new(),
            independent_layer: None,
        }
    }
}

impl<El> Drop for AddChildCache<El> {
    fn drop(&mut self) {
        self.init_content.element_handle.emit_sys(ElementAbandoned);
    }
}
