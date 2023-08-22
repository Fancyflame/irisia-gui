use anyhow::anyhow;
use irisia_backend::skia_safe::{
    canvas::{SaveLayerFlags, SaveLayerRec},
    BlendMode, Canvas, Paint,
};

use self::queue::{Layer, Queue};
use crate::Result;
pub(crate) use rebuild::LayerRebuilder;

mod queue;
pub(crate) mod rebuild;

pub(crate) struct LayerCompositer {
    layers: Queue,
}

impl LayerCompositer {
    pub fn new() -> Self {
        Self {
            layers: Queue::new(),
        }
    }

    pub fn rebuild<'a>(&'a mut self, canvas: &'a mut Canvas) -> LayerRebuilder<'a> {
        self.layers.clear();
        LayerRebuilder::new(self, canvas)
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
                Layer::Extern { layer, matrix } => {
                    canvas.save();
                    canvas.set_matrix(matrix);
                    let result = layer.composite(canvas)?;
                    canvas.restore();
                    result
                }
            }
        }

        canvas.restore();
        Ok(())
    }
}

pub(crate) trait CustomLayer {
    fn composite(&self, canvas: &mut Canvas) -> Result<()>;
}
