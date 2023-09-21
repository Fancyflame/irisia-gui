use anyhow::anyhow;
use irisia_backend::skia_safe::{colors::TRANSPARENT, Canvas};

use super::{LayerCompositer, SharedLayerCompositer};
use crate::Result;

pub struct LayerRebuilder<'a> {
    pub(super) lc: &'a mut LayerCompositer,
    pub(super) canvas: &'a mut Canvas,
    dirty: bool,
}

impl<'a> LayerRebuilder<'a> {
    pub(super) fn new(lc: &'a mut LayerCompositer, canvas: &'a mut Canvas) -> Self {
        canvas.save();
        canvas.reset_matrix();
        Self {
            lc,
            canvas,
            dirty: false,
        }
    }

    pub(crate) fn draw_in_place(&mut self) -> &mut Canvas {
        if self.dirty {
            self.canvas.restore();
        }
        self.dirty = true;
        self.canvas.save();
        self.canvas
    }

    pub(crate) fn new_layer(&mut self, custom_layer: SharedLayerCompositer) -> Result<()> {
        self.flush()?;
        let matrix = self.canvas.local_to_device();
        self.lc.layers.add_layer(custom_layer, matrix);
        self.canvas.clear(TRANSPARENT);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let bitmap = self.lc.layers.add_bitmap(&self.canvas.image_info());
        if !self.canvas.read_pixels_to_bitmap(bitmap, (0, 0)) {
            self.lc.layers.pop();
            return Err(anyhow!("cannot flush canvas content"));
        }
        self.canvas.restore();
        self.dirty = false;

        Ok(())
    }
}

impl Drop for LayerRebuilder<'_> {
    fn drop(&mut self) {
        self.flush().expect("flush at drop time failed");
        self.canvas.clear(TRANSPARENT);
        self.canvas.restore();
    }
}
