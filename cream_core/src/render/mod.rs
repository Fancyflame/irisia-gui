use anyhow::{anyhow, Result};

use crate::{
    element::{Element, NoProps, RcHandle, RenderContent},
    event::{
        event_state::{build::EventListenerBuilder, wrap::WrappedEvents},
        global_register::system_event_register::SystemEventRegister,
        Event,
    },
    primary::Point,
    structure::{add_child::pl_cache::ProxyLayerCache, slot::Slot, EmptyStructure},
    style::NoStyle,
};

pub struct Renderer<El> {
    application: RcHandle<ProxyLayerCache<(), El>>,
    global_listeners: SystemEventRegister,
}

impl<El> Renderer<El>
where
    El: Element<Children<EmptyStructure> = EmptyStructure>,
{
    pub fn emit_event<E: Event>(&mut self, event: &E, point: Option<Point>) {
        self.global_listeners.emit(event, point);
    }

    pub fn redraw(&mut self) -> Result<&[u8]> {
        self.global_listeners.clear();

        self.application.borrow_mut().elem.render(
            Default::default(),
            &NoStyle,
            WrappedEvents::new_empty(),
            EventListenerBuilder::new(&self.application),
            Slot {
                cache: &mut (),
                node: EmptyStructure,
            },
            RenderContent {
                canvas: &mut canvas,
                event_register: &mut self.global_listeners,
                region: (
                    (0, 0).into(),
                    (self.size.0 as u32, self.size.1 as u32).into(),
                ),
            },
        )?;

        if !self.surface.read_pixels(
            &self.output_image_info,
            &mut self.render_buffer,
            self.size.0 as usize,
            (0, 0),
        ) {
            return Err(anyhow!("cannot read pixels from canvas"));
        }

        Ok(&self.render_buffer)
    }
}
