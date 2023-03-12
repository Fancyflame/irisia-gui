use anyhow::{anyhow, Result};
use pixels::{Pixels, SurfaceTexture};
use skia_safe::{Canvas, Color, ColorSpace, ColorType, ImageInfo, Surface};
use winit::dpi::PhysicalSize;

use crate::WinitWindow;

pub struct SurfaceProvider {
    window_pixels: Pixels,
    surface: Surface,
    output_image_info: ImageInfo,
    size: (u32, u32),
    size_needs_update: bool,
}

impl SurfaceProvider {
    pub fn new(window: &WinitWindow) -> Result<Self> {
        let PhysicalSize { width, height } = window.inner_size();

        let pixels = Pixels::new(width, height, SurfaceTexture::new(width, height, window));

        let surface = Surface::new_raster_n32_premul((width as _, height as _))
            .ok_or_else(|| anyhow!("no surface"))?;

        Ok(SurfaceProvider {
            window_pixels: pixels?,
            output_image_info: ImageInfo::new(
                (width as _, height as _),
                ColorType::RGBA8888,
                skia_safe::AlphaType::Opaque,
                Some(ColorSpace::new_srgb()),
            ),
            surface,
            size: (width, height),
            size_needs_update: false,
        })
    }

    pub fn render<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Canvas, (u32, u32)) -> Result<()>,
    {
        self.resize_real()?;

        let canvas = self.surface.canvas();
        canvas.clear(Color::WHITE);
        f(canvas, self.size)?;

        if !self.surface.read_pixels(
            &self.output_image_info,
            self.window_pixels.get_frame_mut(),
            self.size.0 as usize,
            (0, 0),
        ) {
            return Err(anyhow!("cannot read pixels from canvas"));
        }

        self.window_pixels.render()?;

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        self.size_needs_update = true;
    }

    fn resize_real(&mut self) -> Result<()> {
        if !self.size_needs_update {
            return Ok(());
        }

        let (width, height) = self.size;
        let size2x = (
            (width as f32 / 256.0).ceil() as i32,
            (height as f32 / 256.0).ceil() as i32,
        );

        if (self.surface.width(), self.surface.height()) != size2x {
            self.surface =
                Surface::new_raster_n32_premul(size2x).ok_or_else(|| anyhow!("no surface"))?;
        }

        self.window_pixels.resize_buffer(width, height)?;
        self.window_pixels.resize_surface(width, height)?;

        self.output_image_info = self
            .output_image_info
            .with_dimensions((width as _, height as _));

        self.size_needs_update = false;

        Ok(())
    }
}
