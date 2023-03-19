use std::sync::Arc;

use anyhow::{anyhow, Result};
use pixels::{Pixels, SurfaceTexture};
use skia_safe::{Canvas, Color, ColorSpace, ColorType, ImageInfo, Surface};
use winit::dpi::PhysicalSize;

use super::Window;

pub struct SurfaceProvider {
    window: Arc<Window>,
    window_pixels: Pixels,
    surface: Surface,
    output_image_info: ImageInfo,
    size: (u32, u32),
}

impl SurfaceProvider {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let PhysicalSize { width, height } = window.inner_size();

        let pixels = Pixels::new(
            width,
            height,
            SurfaceTexture::new(width, height, &window.winit_window),
        );

        let surface = Surface::new_raster_n32_premul((width as _, height as _))
            .ok_or_else(|| anyhow!("no surface"))?;

        Ok(SurfaceProvider {
            window,
            window_pixels: pixels?,
            output_image_info: ImageInfo::new(
                (width as _, height as _),
                ColorType::RGBA8888,
                skia_safe::AlphaType::Opaque,
                Some(ColorSpace::new_srgb()),
            ),
            surface,
            size: (width, height),
        })
    }

    pub fn render<F>(
        &mut self,
        PhysicalSize { width, height }: PhysicalSize<u32>,
        f: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut Canvas, (u32, u32)) -> Result<()>,
    {
        self.resize_real((width, height))?;
        //let (width, height) = self.size;

        let canvas = self.surface.canvas();
        canvas.clear(Color::YELLOW);
        f(canvas, (width, height))?;

        if !self.surface.read_pixels(
            &self.output_image_info,
            self.window_pixels.get_frame_mut(),
            width as usize * 4,
            (0, 0),
        ) {
            return Err(anyhow!("cannot read pixels from canvas"));
        }

        self.window_pixels.render()?;

        Ok(())
    }

    fn resize_real(&mut self, size: (u32, u32)) -> Result<()> {
        if self.size == size {
            return Ok(());
        }
        println!("resize");

        let (width, height) = size;

        let size2x = (
            (width as f32 / 256.0).ceil() as i32 * 256,
            (height as f32 / 256.0).ceil() as i32 * 256,
        );

        self.window_pixels.resize_surface(width, height)?;
        self.window_pixels.resize_buffer(width, height)?;

        /*println!("start");
        self.window_pixels = Pixels::new(
            width,
            height,
            SurfaceTexture::new(width, height, &self.window.winit_window),
        )?;
        println!("end");*/

        if (self.surface.width(), self.surface.height()) != size2x {
            self.surface = Surface::new_raster_n32_premul((width as _, height as _))
                .ok_or_else(|| anyhow!("no surface"))?;
        }

        self.output_image_info = self
            .output_image_info
            .with_dimensions((width as _, height as _));

        self.size = size;

        Ok(())
    }
}
