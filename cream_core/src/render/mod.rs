use std::{rc::Rc, time::Duration};

use anyhow::Result;
use cream_backend::{window::Window, AppWindow, Canvas, WindowEvent};

use crate::{
    element::{Element, RcHandle, RenderContent},
    event::{
        event_state::{build::EventListenerBuilder, wrap::WrappedEvents},
        global_event_register::SystemEventRegister,
    },
    primary::Point,
    structure::{add_child::pl_cache::ProxyLayerCache, slot::Slot, EmptyStructure},
    style::NoStyle,
};

pub struct Application<El> {
    window: Rc<Window>,
    application: RcHandle<ProxyLayerCache<(), El>>,
    global_listeners: SystemEventRegister,
}

impl<El> AppWindow for Application<El>
where
    El: Element<Children<EmptyStructure> = EmptyStructure>,
{
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()> {
        self.global_listeners.clear();
        let mut cache = self.application.borrow_mut();
        cache.elem.render(
            Default::default(),
            &NoStyle,
            WrappedEvents::new_empty(),
            EventListenerBuilder::new(&self.application),
            Slot {
                node: EmptyStructure,
                cache: &mut (),
            },
            RenderContent {
                canvas,
                event_register: &mut self.global_listeners,
                region: (Point(0, size.0), Point(0, size.1)),
                window: &self.window,
                delta,
            },
        )
    }

    fn on_window_event(&mut self, event: WindowEvent) {
        todo!(); //self.global_listeners.emit(event, point)
    }
}
