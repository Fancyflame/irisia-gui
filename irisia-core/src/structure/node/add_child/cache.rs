use std::sync::Arc;

use anyhow::anyhow;
use tokio::sync::Mutex;

use crate::{
    application::{
        content::GlobalContent,
        event_comp::{NewPointerEvent, NodeEventMgr},
    },
    element::InitContent,
    event::standard::ElementAbandoned,
    primitive::Region,
    structure::{
        cache::NodeCache,
        layer::{LayerRebuilder, SharedLayerCompositer},
        Element,
    },
    Result,
};

pub struct AddChildCache<El, Cc> {
    pub(super) element: Arc<Mutex<El>>,
    pub(super) init_content: InitContent<El>,
    pub(super) children_cache: Cc,
    independent_layer: Option<SharedLayerCompositer>,
    event_mgr: NodeEventMgr,
    pub(super) interact_region: Option<Region>,
    pub(super) draw_region: Region,
}

impl<El, Cc> AddChildCache<El, Cc>
where
    El: Element,
    Cc: Default,
{
    pub(super) fn new<F, Oc>(
        global_content: &GlobalContent,
        draw_region: Region,
        creator: F,
        on_create: Oc,
    ) -> Self
    where
        F: FnOnce(&InitContent<El>, &mut Cc) -> El,
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
            event_mgr: NodeEventMgr::new(),
            interact_region: None,
            draw_region,
        }
    }

    fn render_onto(&self, rebuilder: &mut LayerRebuilder) -> Result<()>
    where
        Cc: NodeCache,
    {
        self.element
            .blocking_lock()
            .render(rebuilder.draw_in_place(), self.draw_region)?;

        self.children_cache.render(rebuilder)?;
        Ok(())
    }
}

impl<El, Cc> Drop for AddChildCache<El, Cc> {
    fn drop(&mut self) {
        self.init_content.event_handle.emit_sys(ElementAbandoned);
    }
}

impl<El, Cc> NodeCache for Option<AddChildCache<El, Cc>>
where
    El: Element,
    Cc: NodeCache,
{
    fn render(&self, rebuilder: &mut LayerRebuilder) -> Result<()> {
        let Some(this) = self else {
            return Err(anyhow!("inner error: this AddChildCache is not initialized"));
        };

        match &this.independent_layer {
            Some(rc) => {
                let mut borrow_mut = rc.borrow_mut();
                let mut rebuilder2 = rebuilder.new_layer(&mut borrow_mut)?;
                this.render_onto(&mut rebuilder2)
            }
            None => this.render_onto(rebuilder),
        }
    }

    fn emit_event(&mut self, new_event: &NewPointerEvent) -> bool {
        let Some(this) = self else {
            return false;
        };

        let children_logically_entered = this.children_cache.emit_event(new_event);
        this.event_mgr
            .update_and_emit(new_event, this.interact_region, children_logically_entered)
    }
}
