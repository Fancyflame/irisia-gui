use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use anyhow::anyhow;
use irisia_backend::skia_safe::{
    canvas::{SaveLayerFlags, SaveLayerRec},
    BlendMode, Canvas, Paint,
};

use self::queue::{Layer, Queue};
use crate::Result;
pub(crate) use rebuild::Rebuilder;

mod queue;
pub(crate) mod rebuild;

pub(crate) type SharedLayerCompositer = Rc<RefCell<LayerCompositer>>;
type WeakLayerCompositer = Weak<RefCell<LayerCompositer>>;

pub(crate) struct LayerCompositer {
    layers: Queue,
    self_weak: WeakLayerCompositer,
}

impl LayerCompositer {
    pub fn new_shared() -> SharedLayerCompositer {
        Rc::new_cyclic(|weak| {
            RefCell::new(Self {
                layers: Queue::new(),
                self_weak: weak.clone(),
            })
        })
    }

    pub fn rebuild<'a>(&'a mut self, canvas: &'a mut Canvas) -> Rebuilder<'a> {
        self.layers.clear();
        Rebuilder::new(self, canvas, false)
    }

    pub fn composite(&self, canvas: &mut Canvas) -> Result<()> {
        let mut paint = Paint::default();
        paint.set_blend_mode(BlendMode::DstOver);
        let rec = SaveLayerRec::default()
            .flags(SaveLayerFlags::INIT_WITH_PREVIOUS)
            .paint(&paint);
        canvas.save_layer(&rec);

        for layer in self.layers.iter() {
            match layer {
                Layer::Normal(bitmap) => {
                    if !canvas.write_pixels_from_bitmap(bitmap, (0, 0)) {
                        return Err(anyhow!("cannot write bitmap to canvas"));
                    }
                }
                Layer::Extern { layer, matrix } => match layer.upgrade() {
                    Some(rc) => {
                        canvas.save();
                        canvas.set_matrix(matrix);
                        let result = rc.borrow().composite(canvas)?;
                        canvas.restore();
                        result
                    }
                    None => {
                        return Err(anyhow!(
                            "child layer has dropped. please DO UPDATE before compositing layers"
                        ))
                    }
                },
            }
        }

        canvas.restore();
        Ok(())
    }
}
