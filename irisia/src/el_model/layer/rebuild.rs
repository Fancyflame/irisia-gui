use std::cell::RefMut;

use anyhow::anyhow;
use irisia_backend::skia_safe::{colors::TRANSPARENT, Canvas, ColorType, IPoint, ISize, Pixmap};

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

    pub(crate) fn append_layer(&mut self, custom_layer: &SharedLayerCompositer) -> Result<()> {
        self.flush()?;
        let matrix = self.canvas.local_to_device();
        self.lc.layers.add_layer(custom_layer.clone(), matrix);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let (rect_start, rect_size) = {
            /*self.canvas
            .peek_pixels()
            .map(min_opaque_rect)
            .unwrap_or_else(|| ((0, 0).into(), self.canvas.image_info().dimensions()))*/
            ((0, 0).into(), self.canvas.image_info().dimensions())
        };

        let bitmap = self.lc.layers.add_bitmap(
            &self.canvas.image_info().with_dimensions(rect_size),
            rect_start,
        );
        bitmap.alloc_pixels();

        if !self.canvas.read_pixels_to_bitmap(bitmap, rect_start) {
            self.lc.layers.pop();
            return Err(anyhow!("cannot flush canvas content"));
        }
        self.dirty = false;

        Ok(())
    }
}

fn _min_opaque_rect(pixmap: Pixmap) -> (IPoint, ISize) {
    debug_assert_eq!(pixmap.color_type(), ColorType::RGBA8888);
    let mut pixels = pixmap.pixels::<[u8; 4]>().unwrap();
    let row_length = pixmap.row_bytes_as_pixels();
    let column_length = pixels.len() / row_length;

    let mut start = (0usize, 0i32);
    let mut end = (row_length, column_length as i32);

    // move start Y dimension
    while !pixels.is_empty() {
        let (row, rest) = pixels.split_at(row_length);
        if row.iter().any(|arr| arr[3] != 0) {
            break;
        }
        start.1 += 1;
        pixels = rest;
    }

    // move end Y dimension
    while !pixels.is_empty() {
        let (rest, row) = pixels.split_at(pixels.len() - row_length);
        if row.iter().any(|arr| arr[3] != 0) {
            break;
        }
        end.1 -= 1;
        pixels = rest;
    }

    // move start X dimension
    while !pixels.is_empty() {
        if pixels
            .chunks_exact(row_length)
            .any(|chunk| chunk[start.0][3] != 0)
        {
            break;
        }
        start.0 += 1;
    }

    // move end X dimension
    while !pixels.is_empty() {
        if pixels
            .chunks_exact(row_length)
            .any(|chunk| chunk[end.0 - 1][3] != 0)
        {
            break;
        }
        end.0 -= 1
    }

    if pixels.is_empty() {
        ((0, 0).into(), (0, 0).into())
    } else {
        let width = (end.0 - start.0) as i32;
        let height = end.1 - start.1;
        (
            IPoint::new(start.0 as i32, start.1),
            ISize::new(width, height),
        )
    }
}

impl Drop for LayerRebuilder<'_> {
    fn drop(&mut self) {
        self.flush().expect("flush at drop time failed");
        self.canvas.clear(TRANSPARENT);
        self.canvas.restore();
    }
}
