use anyhow::{anyhow, Result};
use skia_safe::{ColorSpace, ColorType, ImageInfo, Surface};

use crate::{
    element::{Element, NoProps, RcHandle, RenderContent},
    event::{
        event_state::{build::EvlBuilder, wrap::WrappedEvents},
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
    surface: Surface,
    output_image_info: ImageInfo,
    size: (i32, i32),
    render_buffer: Vec<u8>,
}

impl<El> Renderer<El>
where
    El: Element<Children<EmptyStructure> = EmptyStructure, Props<'static> = NoProps>,
{
    pub fn new(width: i32, height: i32) -> Self {
        let surface = Surface::new_raster_n32_premul((width, height)).expect("no surface!");

        Renderer {
            application: Default::default(),
            global_listeners: SystemEventRegister::new(),
            output_image_info: ImageInfo::new(
                (width, height),
                ColorType::RGBA8888,
                skia_safe::AlphaType::Opaque,
                Some(ColorSpace::new_srgb()),
            ),
            surface,
            size: (width, height),
            render_buffer: Vec::with_capacity((width * height * 4) as usize),
        }
    }

    pub fn emit_event<E: Event>(&mut self, event: &E, point: Option<Point>) {
        self.global_listeners.emit(event, point);
    }

    pub fn redraw(&mut self) -> Result<&[u8]> {
        let mut canvas = self.surface.canvas();
        self.global_listeners.clear();

        self.application.borrow_mut().elem.render(
            NoProps {},
            &NoStyle,
            WrappedEvents::new_empty(),
            EvlBuilder::new(&self.application),
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
            return Err(anyhow!("Cannot read pixels from canvas"));
        }

        Ok(&self.render_buffer)
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.size = (width, height);
        self.surface = Surface::new_raster_n32_premul((width, height)).expect("no surface!");
        self.output_image_info = ImageInfo::new(
            (width, height),
            ColorType::RGBA8888,
            skia_safe::AlphaType::Opaque,
            Some(ColorSpace::new_srgb()),
        );
        self.render_buffer.resize((width * height * 4) as _, 0);
    }
}
