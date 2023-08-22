use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use irisia_backend::{skia_safe::Canvas, WinitWindow};

use crate::{
    dom::layer::{LayerCompositer, LayerRebuilder},
    Result,
};

use self::list::RedrawList;

pub(crate) mod list;

pub(crate) struct RedrawScheduler {
    independent_layers: HashMap<usize, Rc<dyn RedrawObject>>,
    root_layer: LayerCompositer,
}

pub(crate) const ROOT_LAYER_ID: LayerId = LayerId(0);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LayerId(usize);

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> (Self, RedrawList) {
        (
            RedrawScheduler {
                independent_layers: HashMap::new(),
                root_layer: LayerCompositer::new(),
            },
            RedrawList {
                window,
                list: HashSet::new(),
                redraw_req_sent: false,
            },
        )
    }

    pub fn reg(&mut self, value: Rc<dyn RedrawObject>) -> LayerId {
        let key = Rc::as_ptr(&value).cast::<()>() as usize;
        assert_ne!(key, 0);
        let not_exists = self.independent_layers.insert(key, value).is_none();
        debug_assert!(not_exists);
        LayerId(key)
    }

    pub fn del(&mut self, key: LayerId) {
        let already_exists = self.independent_layers.remove(&key.0).is_some();
        debug_assert!(already_exists);
    }

    pub fn redraw(
        &mut self,
        canvas: &mut Canvas,
        mut root_element_renderer: impl FnMut(&mut LayerRebuilder, Duration) -> Result<()>,
        interval: Duration,
        list: &mut RedrawList,
    ) -> Result<()> {
        if list.list.is_empty() {
            return Ok(());
        }

        let mut errors = Vec::new();

        for ptr in list.list.drain() {
            let result = if ptr == ROOT_LAYER_ID {
                root_element_renderer(&mut self.root_layer.rebuild(canvas), interval)
            } else {
                match self.independent_layers.get(&ptr.0) {
                    Some(ro) => ro.redraw(canvas, interval),
                    None => Err(anyhow!("redraw object not registered")),
                }
            };

            if let Err(err) = result {
                errors.push(err);
            }
        }

        list.redraw_req_sent = false;
        fmt_errors(&errors)
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

pub trait RedrawObject {
    fn redraw(&self, canvas: &mut Canvas, interval: Duration) -> Result<()>;
}
