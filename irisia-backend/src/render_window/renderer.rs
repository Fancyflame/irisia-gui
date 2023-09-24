#[cfg(feature = "fps_recorder")]
use std::{
    sync::{atomic::AtomicU16, Arc},
    time::Duration,
};

use anyhow::{anyhow, Result};
use pixels::{
    wgpu::{BlendState, DeviceDescriptor, Features, Limits},
    Pixels, PixelsBuilder, SurfaceTexture,
};
use skia_safe::{Canvas, Color, ColorSpace, ColorType, ImageInfo, Surface};
use winit::dpi::PhysicalSize;

use crate::WinitWindow;

pub struct Renderer {
    window_pixels: Pixels,
    surface: Surface,
    output_image_info: ImageInfo,
    size: PhysicalSize<u32>,
    size2x: (u32, u32),
    #[cfg(feature = "fps_recorder")]
    counter: Arc<AtomicU16>,
}

#[cfg(feature = "fps_recorder")]
fn fps_recorder() -> Arc<AtomicU16> {
    let counter = Arc::new(AtomicU16::new(0));
    let counter_cloned = counter.clone();

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(2));
        println!(
            "{}fps",
            counter.swap(0, std::sync::atomic::Ordering::Relaxed) / 2
        );
    });
    counter_cloned
}

impl Renderer {
    pub fn new(window: &Arc<WinitWindow>) -> Result<Self> {
        let PhysicalSize { width, height } = window.inner_size();

        let (w2x, h2x) = to_size2x(window.inner_size());

        let pixels =
            PixelsBuilder::new(width, height, SurfaceTexture::new(width, height, &**window))
                .blend_state(BlendState::REPLACE)
                .enable_vsync(false)
                .device_descriptor(DeviceDescriptor {
                    label: Default::default(),
                    features: Features::empty(),
                    limits: Limits::downlevel_defaults().using_resolution(Limits::default()),
                })
                .clear_color(pixels::wgpu::Color::WHITE)
                .build()?;

        let image_info = ImageInfo::new(
            (width as _, height as _),
            ColorType::RGBA8888,
            skia_safe::AlphaType::Opaque,
            Some(ColorSpace::new_srgb()),
        );

        let surface = skia_safe::surfaces::raster_n32_premul((w2x as _, h2x as _))
            .ok_or_else(|| anyhow!("skia surface not found"))?;

        Ok(Renderer {
            window_pixels: pixels,
            output_image_info: image_info,
            surface,
            size: window.inner_size(),
            size2x: (w2x, h2x),

            #[cfg(feature = "fps_recorder")]
            counter: fps_recorder(),
        })
    }

    pub fn render<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Canvas) -> Result<()>,
    {
        let canvas = self.surface.canvas();
        canvas.clear(Color::WHITE);
        f(canvas)?;

        if !self.surface.read_pixels(
            &self.output_image_info,
            self.window_pixels.frame_mut(),
            (self.size.width as usize) * 4,
            (0, 0),
        ) {
            return Err(anyhow!("cannot read pixels from canvas"));
        }

        self.window_pixels.render()?;

        #[cfg(feature = "fps_recorder")]
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        if self.size == size || size.width == 0 || size.height == 0 {
            return Ok(());
        }

        let PhysicalSize { width, height } = size;

        let size2x = to_size2x(size);

        self.window_pixels.resize_buffer(width, height)?;
        self.window_pixels.resize_surface(width, height)?;

        if size2x != self.size2x {
            self.size2x = size2x;
            self.surface = self
                .surface
                .new_surface_with_dimensions((size2x.0 as _, size2x.1 as _))
                .ok_or_else(|| anyhow!("skia surface not found"))?;
        }

        self.output_image_info = self
            .output_image_info
            .with_dimensions((width as _, height as _));

        self.size = size;

        Ok(())
    }
}

fn to_size2x(size: PhysicalSize<u32>) -> (u32, u32) {
    let PhysicalSize { width, height } = size;
    (
        (width as f32 / 256.0).ceil() as u32 * 256,
        (height as f32 / 256.0).ceil() as u32 * 256,
    )
}
