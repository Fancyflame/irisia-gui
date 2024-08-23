use irisia_backend::skia_safe::{Bitmap, IPoint, ImageInfo, M44};
use smallvec::SmallVec;

use super::SharedLayerCompositer;

pub(super) enum Layer {
    Bitmap {
        bitmap: Bitmap,
        offset: IPoint,
    },
    CacheLayer {
        layer: SharedLayerCompositer,
        matrix: M44,
    },
}

pub(super) struct Queue {
    buffer: SmallVec<[Layer; 1]>,
    len: usize,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            buffer: SmallVec::new(),
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn add_bitmap(&mut self, image_info: &ImageInfo, offset: IPoint) -> &mut Bitmap {
        loop {
            match self.buffer.get_mut(self.len) {
                None => {
                    let mut bitmap = Bitmap::new();
                    assert!(bitmap.set_info(image_info, None));
                    self.buffer.push(Layer::Bitmap { bitmap, offset });
                    break;
                }
                Some(Layer::Bitmap {
                    bitmap,
                    offset: old_offset,
                }) => {
                    assert!(bitmap.set_info(image_info, None));
                    *old_offset = offset;
                    break;
                }
                Some(Layer::CacheLayer { .. }) => {
                    self.buffer.swap_remove(self.len);
                }
            }
        }

        self.len += 1;
        match self.buffer.last_mut() {
            Some(Layer::Bitmap { bitmap, .. }) => bitmap,
            _ => unreachable!(),
        }
    }

    pub fn add_layer(&mut self, layer: SharedLayerCompositer, matrix: M44) {
        let layer = Layer::CacheLayer { layer, matrix };

        match self.buffer.get_mut(self.len) {
            Some(ext @ Layer::CacheLayer { .. }) => *ext = layer,
            Some(normal @ Layer::Bitmap { .. }) => {
                let bitmap = std::mem::replace(normal, layer);
                self.buffer.push(bitmap);
            }
            None => {
                self.buffer.push(layer);
            }
        }
        self.len += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.buffer.iter().take(self.len)
    }

    pub fn pop(&mut self) {
        if let Some(len) = self.len.checked_sub(1) {
            self.len = len;
        }
    }
}
