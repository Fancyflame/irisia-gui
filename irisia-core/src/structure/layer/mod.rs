use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;
use irisia_backend::skia_safe::{
    canvas::{SaveLayerFlags, SaveLayerRec},
    BlendMode, Canvas, Paint,
};

use self::queue::{Layer, Queue};
use crate::Result;
pub(crate) use rebuild::Rebuild;

mod queue;
pub(crate) mod rebuild;

pub(crate) type RcLc = Rc<RefCell<LayerCompositer>>;

pub(crate) struct LayerCompositer {
    layers: Queue,
}

impl LayerCompositer {
    pub fn new_rc() -> RcLc {
        Rc::new(RefCell::new(Self {
            layers: Queue::new(),
        }))
    }

    pub fn rebuild<'a>(&'a mut self, canvas: &'a mut Canvas) -> Rebuild<'a> {
        self.layers.clear();
        canvas.save();
        Rebuild {
            lc: self,
            canvas,
            dirty: false,
        }
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