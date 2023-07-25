use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    application::content::GlobalContent,
    element::InitContent,
    event::standard::ElementAbandoned,
    structure::{layer::SharedLayerCompositer, Element},
};

pub struct AddChildCache<El, Cc> {
    pub(super) element: Arc<Mutex<El>>,
    pub(super) init_content: InitContent<El>,
    pub(super) children_cache: Cc,
    pub(super) independent_layer: Option<SharedLayerCompositer>,
}

impl<El, Cc> AddChildCache<El, Cc> {
    pub(super) fn new<F, Oc>(global_content: &GlobalContent, creator: F, on_create: Oc) -> Self
    where
        F: FnOnce(&InitContent<El>, &mut Cc) -> El,
        El: Element,
        Cc: Default + 'static,
        Oc: FnOnce(&InitContent<El>),
    {
        let mut children_cache = Cc::default();
        let mut init_content = None;

        let element = Arc::new_cyclic(|weak| {
            let ic = global_content.build_init_content(weak.clone());
            let el = creator(&ic, &mut children_cache);
            init_content = Some(ic);
            Mutex::new(el)
        });

        let init_content = init_content.unwrap();
        on_create(&init_content);

        AddChildCache {
            element,
            init_content,
            children_cache,
            independent_layer: None,
        }
    }
}

impl<El, Cc> Drop for AddChildCache<El, Cc> {
    fn drop(&mut self) {
        self.init_content.event_handle.emit_sys(ElementAbandoned);
    }
}
