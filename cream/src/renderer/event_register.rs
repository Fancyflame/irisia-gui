use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    event::{Event, EventFlow},
    map_rc::{MapRc, MapWeak},
    primary::{Area, Vec2},
    structure::Element,
};

use super::layer::Layer;

pub struct BubbleEventRegister {
    // dyn Any -> 
    listeners: HashMap<TypeId, Vec<MapWeak<(Area, MapRc<dyn Any>)>>>,
}

impl BubbleEventRegister {
    pub fn new() -> Self {
        BubbleEventRegister {
            listeners: HashMap::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        for vec in self.listeners.iter_mut() {
            vec.1.clear();
        }
    }

    pub(crate) fn register<E: Event, El: Element>(&mut self, _: E, lh: &Layer<El>) {
        let type_id = TypeId::of::<E>();
        let listener = MapRc::downgrade(lh);

        match self.listeners.get_mut(&type_id) {
            Some(vec) => vec.push(listener),
            None => {
                let mut vec = Vec::new();
                vec.push(listener);
                self.listeners.insert(type_id, vec);
            }
        }
    }

    pub fn call<E: Event>(&self, ev: E, point: Vec2, arg: &E::Arg) {
        let type_id = TypeId::of::<E>();

        let listeners = match self.listeners.get(&type_id) {
            Some(l) => l,
            None => return,
        };

        let mut is_exact = true;
        for cell in listeners.iter() {
            let cell_layer = match cell.upgrade() {
                Some(n) => n,
                None => continue,
            };
            let layer = (*cell_layer).borrow_mut();

            if point.abs_ge(layer.area().0) && point.abs_le(layer.area().1) {
                let callbacks = layer.callback_list(ev);

                let mut flow = EventFlow {
                    is_exact,
                    bubble: false,
                };

                is_exact = false;

                for cb in callbacks.iter() {
                    cb.call(arg, &mut flow);
                }

                if !flow.bubble {
                    break;
                }
            }
        }
    }
}
