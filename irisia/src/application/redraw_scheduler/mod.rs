use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use irisia_backend::{skia_safe::Canvas, WinitWindow};

use crate::{
    dom::layer::{LayerCompositer, LayerRebuilder},
    Result,
};

pub(crate) use self::{list::RedrawList, register::IndepLayerRegister};

mod list;
mod register;
pub(crate) const ROOT_LAYER_ID: LayerId = LayerId(0);

pub(super) struct RedrawScheduler {
    root_layer_compositer: LayerCompositer,
    register: IndepLayerRegister,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LayerId(usize);

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> (Self, RedrawList) {
        (
            RedrawScheduler {
                register: IndepLayerRegister::new(),
                root_layer_compositer: LayerCompositer::new(),
            },
            RedrawList::new(window),
        )
    }

    pub fn redraw(
        &mut self,
        canvas: &mut Canvas,
        mut root_element_renderer: impl FnMut(
            &mut LayerRebuilder,
            &mut IndepLayerRegister,
            Duration,
        ) -> Result<()>,
        interval: Duration,
        list: &mut RedrawList,
    ) -> Result<()> {
        let mut errors = Vec::new();

        for ptr in list.drain() {
            let result = if ptr == ROOT_LAYER_ID {
                root_element_renderer(
                    &mut self.root_layer_compositer.rebuild(canvas),
                    &mut self.register,
                    interval,
                )
            } else {
                match self.register.get(ptr) {
                    Some(ro) => ro.clone().redraw(canvas, &mut self.register, interval),
                    None => Err(anyhow!("redraw object not registered")),
                }
            };

            if let Err(err) = result {
                errors.push(err);
            }
        }

        fmt_errors(&errors)
    }

    pub fn composite(&self, canvas: &mut Canvas) -> Result<()> {
        self.root_layer_compositer.composite(canvas)
    }
}

fn fmt_errors(errors: &[anyhow::Error]) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    for (index, err) in errors.iter().enumerate() {
        msg += &format!("`{err}`");
        if index != errors.len() - 1 {
            msg += ", ";
        }
    }

    Err(anyhow!(
        "{} error(s) occurred on redraw: {}",
        errors.len(),
        msg
    ))
}

pub(crate) trait RedrawObject {
    fn redraw(&self, canvas: &mut Canvas, interval: Duration) -> Result<()>;
}
