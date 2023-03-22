use anyhow::{anyhow, Result};
use pixels::{Pixels, SurfaceTexture};
use skia_safe::{Canvas, Color, ColorSpace, ColorType, ImageInfo, Surface};
use winit::dpi::PhysicalSize;

use crate::WinitWindow;

pub struct Renderer {
    window_pixels: Pixels,
    surface: Surface,
    output_image_info: ImageInfo,
    size: PhysicalSize<u32>,
}

impl Renderer {
    pub fn new(window: &WinitWindow) -> Result<Self> {
        let PhysicalSize { width, height } = window.inner_size();

        let pixels = Pixels::new(width, height, SurfaceTexture::new(width, height, &window));

        let surface = Surface::new_raster_n32_premul((width as _, height as _))
            .ok_or_else(|| anyhow!("no surface"))?;

        Ok(Renderer {
            window_pixels: pixels?,
            output_image_info: ImageInfo::new(
                (width as _, height as _),
                ColorType::RGBA8888,
                skia_safe::AlphaType::Opaque,
                Some(ColorSpace::new_srgb()),
            ),
            surface,
            size: window.inner_size(),
        })
    }

    pub fn render<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Canvas, (u32, u32)) -> Result<()>,
    {
        let PhysicalSize { width, height } = self.size;
        let canvas = self.surface.canvas();
        canvas.clear(Color::WHITE);
        f(canvas, (width, height))?;

        if !self.surface.read_pixels(
            &self.output_image_info,
            self.window_pixels.frame_mut(),
            (self.size.width as usize) * 4,
            (0, 0),
        ) {
            return Err(anyhow!("cannot read pixels from canvas"));
        }

        self.window_pixels.render()?;

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        if self.size == size {
            return Ok(());
        }

        let PhysicalSize { width, height } = size;

        let size2x = (
            (width as f32 / 256.0).ceil() as i32 * 256,
            (height as f32 / 256.0).ceil() as i32 * 256,
        );

        self.window_pixels.resize_surface(width, height)?;
        self.window_pixels.resize_buffer(width as _, height as _)?;

        if (self.surface.width(), self.surface.height()) != size2x {
            self.surface = Surface::new_raster_n32_premul((size2x.0, size2x.1))
                .ok_or_else(|| anyhow!("no surface"))?;
        }

        self.output_image_info = self
            .output_image_info
            .with_dimensions((width as _, height as _));

        self.size = size;

        Ok(())
    }
}
