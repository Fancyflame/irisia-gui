use anyhow::anyhow;
use irisia_backend::skia_safe::{colors::TRANSPARENT, Canvas};

use super::{LayerCompositer, RcLc};
use crate::Result;

pub(crate) struct Rebuild<'a> {
    pub(super) lc: &'a mut LayerCompositer,
    pub(super) canvas: &'a mut Canvas,
    pub(super) dirty: bool,
}

impl Rebuild<'_> {
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

    pub fn new_layer<F, R>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Canvas) -> Result<RcLc>,
    {
        if self.dirty {
            let bitmap = self.lc.layers.add_bitmap(&self.canvas.image_info());
            if !self.canvas.read_pixels_to_bitmap(bitmap, (0, 0)) {
                self.lc.layers.pop();
                return Err(anyhow!("cannot save layer"));
            }
            self.dirty = false;
        }

        let matrix = self.canvas.local_to_device();
        self.canvas.save();
        self.canvas.clear(TRANSPARENT);
        self.canvas.reset_matrix();
        let result = f(self.canvas);
        self.canvas.clear(TRANSPARENT);
        self.canvas.restore();

        self.lc.layers.add_layer(&result?, matrix);
        Ok(())
    }
}

impl Drop for Rebuild<'_> {
    fn drop(&mut self) {
        self.canvas.restore();
        if !self.dirty {
            return;
        }
        assert!(self
            .canvas
            .read_pixels_to_bitmap(self.lc.layers.add_bitmap(&self.canvas.image_info()), (0, 0)));
    }
}
