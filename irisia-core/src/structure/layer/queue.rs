use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use irisia_backend::skia_safe::{Bitmap, ImageInfo, M44};
use smallvec::SmallVec;

use super::{LayerCompositer, RcLc};

pub(super) enum Layer {
    Normal(Bitmap),
    Extern {
        layer: Weak<RefCell<LayerCompositer>>,
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

    pub fn add_bitmap(&mut self, image_info: &ImageInfo) -> &mut Bitmap {
        loop {
            match self.buffer.get_mut(self.len) {
                None => {
                    let mut bitmap = Bitmap::new();
                    assert!(bitmap.set_info(image_info, None));
                    self.buffer.push(Layer::Normal(bitmap));
                    break;
                }
                Some(Layer::Normal(bitmap)) => {
                    assert!(bitmap.set_info(image_info, None));
                    break;
                }
                Some(Layer::Extern { .. }) => {
                    self.buffer.swap_remove(self.len);
                }
            }
        }

        self.len += 1;
        match self.buffer.last_mut() {
            Some(Layer::Normal(bitmap)) => bitmap,
            _ => unreachable!(),
        }
    }

    pub fn add_layer(&mut self, layer: &RcLc, matrix: M44) {
        let layer = Layer::Extern {
            layer: Rc::downgrade(layer),
            matrix,
        };

        match self.buffer.get_mut(self.len) {
            Some(ext @ Layer::Extern { .. }) => *ext = layer,
            Some(normal @ Layer::Normal { .. }) => {
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

    pub fn last_mut(&mut self) -> Option<&mut Layer> {
        if self.len == 0 {
            None
        } else {
            Some(&mut self.buffer[self.len - 1])
        }
    }

    pub fn pop(&mut self) {
        if let Some(len) = self.len.checked_sub(1) {
            self.len = len;
        }
    }
}
