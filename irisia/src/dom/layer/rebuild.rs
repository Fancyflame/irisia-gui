use std::cell::RefMut;

use anyhow::anyhow;
use irisia_backend::skia_safe::{colors::TRANSPARENT, Canvas};

use super::{LayerCompositer, SharedLayerCompositer};
use crate::Result;

pub struct LayerRebuilder<'a> {
    lc: RefMut<'a, LayerCompositer>,
    canvas: &'a Canvas,
    dirty: bool,
}

impl<'a> LayerRebuilder<'a> {
    pub(super) fn new(lc: &'a SharedLayerCompositer, canvas: &'a Canvas) -> Self {
        canvas.save();
        canvas.clear(TRANSPARENT);
        canvas.reset_matrix();

        let mut lc = lc.borrow_mut();
        lc.layers.clear();

        Self {
            lc,
            canvas,
            dirty: false,
        }
    }

    pub fn canvas(&mut self) -> &Canvas {
        self.dirty = true;
        self.canvas
    }

    pub(crate) fn new_layer<'b>(
        &'b mut self,
        custom_layer: &'b SharedLayerCompositer,
    ) -> Result<LayerRebuilder<'b>> {
        self.flush()?;
        let matrix = self.canvas.local_to_device();
        self.lc.layers.add_layer(custom_layer.clone(), matrix);
        Ok(LayerRebuilder::new(custom_layer, self.canvas))
    }

    fn flush(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let bitmap = self.lc.layers.add_bitmap(&self.canvas.image_info());
        bitmap.alloc_pixels();
        if !self.canvas.read_pixels_to_bitmap(bitmap, (0, 0)) {
            self.lc.layers.pop();
            return Err(anyhow!("cannot flush canvas content"));
        }
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
