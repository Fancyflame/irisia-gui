use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;
use irisia_backend::skia_safe::{
    canvas::{SaveLayerFlags, SaveLayerRec},
    BlendMode, Canvas, Paint,
};

use self::queue::{Layer, Queue};
use crate::Result;
pub use rebuild::LayerRebuilder;

mod queue;
pub(crate) mod rebuild;

pub(crate) type SharedLayerCompositer = Rc<RefCell<LayerCompositer>>;

pub(crate) struct LayerCompositer {
    layers: Queue,
}

impl LayerCompositer {
    pub fn new() -> SharedLayerCompositer {
        Rc::new(RefCell::new(Self {
            layers: Queue::new(),
        }))
    }

    pub fn rebuild<'a>(this: &'a SharedLayerCompositer, canvas: &'a Canvas) -> LayerRebuilder<'a> {
        LayerRebuilder::new(this, canvas)
    }

    pub fn composite(&self, canvas: &Canvas) -> Result<()> {
        let mut paint = Paint::default();
        paint.set_blend_mode(BlendMode::DstOver);
        let rec = SaveLayerRec::default()
            .flags(SaveLayerFlags::INIT_WITH_PREVIOUS)
            .paint(&paint);

        for layer in self.layers.iter() {
            match layer {
                Layer::Normal(bitmap) => {
                    canvas.save_layer(&rec);
                    if !canvas.write_pixels_from_bitmap(bitmap, (0, 0)) {
                        return Err(anyhow!("cannot write bitmap to canvas"));
                    }
                    canvas.restore();
                }
                Layer::Extern { layer, matrix } => {
                    canvas.save();
                    canvas.set_matrix(matrix);
                    layer.borrow().composite(canvas)?;
                    canvas.restore();
                }
            }
        }
        Ok(())
    }
}
