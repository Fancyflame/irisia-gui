use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    element::{render_content::BareContent, ElementHandle, InitContent},
    event::{event_dispatcher::EventDispatcher, standard::ElementAbandoned},
    structure::{cache_box::CacheBox, Element, StructureBuilder},
};

pub struct AddChildCache<El, Pr> {
    pub(super) element: Arc<Mutex<El>>,
    pub(super) init_content: InitContent<El>,
    pub(super) cache_box: CacheBox,
    pub(super) props: Pr,
}

impl<El, Pr> AddChildCache<El, Pr>
where
    Pr: Default,
{
    pub(super) fn new<Sb, F>(content: &BareContent, on_create: F) -> Self
    where
        El: Element<Sb, Props = Pr>,
        Sb: StructureBuilder,
        F: FnOnce(&InitContent<El>),
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

            let el = El::create(&ic);
            init_content = Some(ic);
            Mutex::new(el)
        });
        let init_content = init_content.unwrap();
        on_create(&init_content);

        AddChildCache {
            element,
            init_content,
            cache_box: CacheBox::new(),
            props: Pr::default(),
        }
    }
}

impl<El, Pr> Drop for AddChildCache<El, Pr> {
    fn drop(&mut self) {
        self.init_content.element_handle.emit_sys(ElementAbandoned);
    }
}
