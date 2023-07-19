use anyhow::anyhow;
use irisia_backend::skia_safe::{colors::TRANSPARENT, Canvas};

use super::LayerCompositer;
use crate::Result;

pub(crate) struct Rebuilder<'a> {
    pub(super) lc: &'a mut LayerCompositer,
    pub(super) canvas: &'a mut Canvas,
    dirty: bool,
    has_parent: bool,
}

impl<'a> Rebuilder<'a> {
    pub(super) fn new(
        lc: &'a mut LayerCompositer,
        canvas: &'a mut Canvas,
        has_parent: bool,
    ) -> Self {
        canvas.save();
        canvas.reset_matrix();
        Self {
            lc,
            canvas,
            dirty: false,
            has_parent,
        }
    }

    pub fn draw_in_place<F, R>(&mut self, draw: F) -> Result<R>
    where
        F: FnOnce(&mut Canvas) -> Result<R>,
    {
        self.dirty = true;
        self.canvas.save();
        let r = draw(self.canvas);
        self.canvas.restore();
        r
    }

    pub fn new_layer<'b>(&'b mut self, lc: &'b mut LayerCompositer) -> Result<Rebuilder<'b>> {
        if self.dirty {
            let bitmap = self.lc.layers.add_bitmap(&self.canvas.image_info());
            if !self.canvas.read_pixels_to_bitmap(bitmap, (0, 0)) {
                self.lc.layers.pop();
                return Err(anyhow!("cannot save layer"));
            }
            self.dirty = false;
        }

        let matrix = self.canvas.local_to_device();
        self.lc.layers.add_layer(lc.self_weak.clone(), matrix);
        self.canvas.clear(TRANSPARENT);
        Ok(Rebuilder::new(lc, self.canvas, true))
    }
}

impl Drop for Rebuilder<'_> {
    fn drop(&mut self) {
        if self.dirty {
            assert!(self.canvas.read_pixels_to_bitmap(
                self.lc.layers.add_bitmap(&self.canvas.image_info()),
                (0, 0)
            ));
        }

        if self.has_parent {
            self.canvas.clear(TRANSPARENT);
        }

        self.canvas.restore();
    }
}
