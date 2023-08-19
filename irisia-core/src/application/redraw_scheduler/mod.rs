use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use anyhow::anyhow;
use irisia_backend::{skia_safe::Canvas, WinitWindow};

use crate::Result;

use self::list::RedrawList;

pub(crate) mod list;

pub struct RedrawScheduler {
    independent_layers: HashMap<usize, Rc<dyn RedrawObject>>,
}

#[derive(Clone)]
pub struct LayerId(usize);

impl RedrawScheduler {
    pub fn new(window: Arc<WinitWindow>) -> (Self, RedrawList) {
        (
            RedrawScheduler {
                independent_layers: HashMap::new(),
            },
            RedrawList {
                window,
                list: Vec::new(),
                redraw_req_sent: false,
            },
        )
    }

    pub fn reg(&mut self, value: Rc<dyn RedrawObject>) -> LayerId {
        let key = Rc::as_ptr(&value).cast::<()>() as usize;
        let not_exists = self.independent_layers.insert(key, value).is_none();
        debug_assert!(not_exists);
        LayerId(key)
    }

    pub fn del(&mut self, key: LayerId) {
        let already_exists = self.independent_layers.remove(&key.0).is_some();
        debug_assert!(already_exists);
    }

    pub fn redraw(
        &self,
        canvas: &mut Canvas,
        interval: Duration,
        list: &mut RedrawList,
    ) -> Result<()> {
        if list.list.is_empty() {
            return Ok(());
        }

        let mut errors = Vec::new();

        for ptr in list.list.drain(..) {
            let Some(ro) = self.independent_layers.get(&ptr.0)
            else {
                errors.push(anyhow!("redraw object not registered"));
                continue;
            };

            if let Err(err) = ro.redraw(canvas, interval) {
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
