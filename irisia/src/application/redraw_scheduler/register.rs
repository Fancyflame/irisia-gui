use std::{collections::HashMap, rc::Rc};

use crate::application::redraw_scheduler::LayerId;

use super::RedrawObject;

pub struct IndepLayerRegister(HashMap<usize, Rc<dyn RedrawObject>>);

impl IndepLayerRegister {
    pub(super) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn reg(&mut self, value: Rc<dyn RedrawObject>) -> LayerId {
        let key = Rc::as_ptr(&value).cast::<()>() as usize;
        assert_ne!(key, 0);
        let not_exists = self.0.insert(key, value).is_none();
        debug_assert!(not_exists);
        LayerId(key)
    }

    pub(crate) fn del(&mut self, key: LayerId) {
        let already_exists = self.0.remove(&key.0).is_some();
        debug_assert!(already_exists);
    }

    pub(super) fn get(&self, key: LayerId) -> Option<&Rc<dyn RedrawObject>> {
        self.0.get(&key.0)
    }
}
