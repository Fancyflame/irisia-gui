use std::{any::TypeId, collections::HashMap};

use crate::{
    event::{Event, EventFlow},
    map_rc::MapWeak,
    primary::Vec2,
    structure::Element,
};

use super::layer::LayerTrait;

pub struct BubbleEventRegister {
    // `usize`s is the indexes of layers which listen this event
    map: HashMap<TypeId, Vec<usize>>,
}

impl BubbleEventRegister {
    pub fn new() -> Self {
        BubbleEventRegister {
            map: HashMap::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        for vec in self.map.iter_mut() {
            vec.1.clear();
        }
    }

    pub(crate) fn register<E: Event, El: Element>(&mut self, ev: E, index: usize) {
        let type_id = ev.type_id();
        match self.map.get_mut(&type_id) {
            Some(vec) => {
                vec.push(index);
            }
            None => {
                self.map.insert(type_id, vec![index]);
            }
        }
    }

    pub(crate) fn call<E: Event>(
        &self,
        ev: E,
        arg: &E::Arg,
        point: Vec2,
        layers: &[MapWeak<dyn LayerTrait>],
    ) {
        let type_id = TypeId::of::<E>();

        let indexes = match self.map.get(&type_id) {
            Some(i) => &*i,
            None => return,
        };

        let mut is_exact = true;
        for index in indexes {
            let layer = layers[*index].upgrade().unwrap();
            let attrs = layer.attrs();

            if point.abs_ge(attrs.area().0) && point.abs_le(attrs.area().1) {
                let callbacks = match attrs.event_target().get(ev) {
                    Some(cb) => cb,
                    None => unreachable!("The event is expected exists in this event target"),
                };

                let mut flow = EventFlow {
                    is_exact,
                    bubble: false,
                };

                is_exact = false;

                for cb in &*callbacks.borrow() {
                    cb.upgrade().unwrap().call(arg, &mut flow);
                }

                if !flow.bubble() {
                    break;
                }
            }
        }
    }
}
