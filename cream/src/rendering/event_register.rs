use std::{any::TypeId, collections::HashMap};

use crate::{
    data_driven::Watchable,
    event::{Event, EventFlow},
    primary::Vec2,
    structure::element::ElementHandle,
};

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

    pub(crate) fn register_unchecked(&mut self, type_id: TypeId, index: usize) {
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
        args: &E::Arg,
        point: Vec2,
        layers: &[ElementHandle],
    ) {
        let type_id = TypeId::of::<E>();

        let indexes = match self.map.get(&type_id) {
            Some(i) => &*i,
            None => return,
        };

        let mut is_exact = true;
        for index in indexes {
            let layer = layers[*index].borrow();
            let svc = layer.service();
            let area = *svc.area().get();

            if point.abs_ge(area.0) && point.abs_le(area.1) {
                let mut flow = EventFlow {
                    is_exact,
                    bubble: false,
                };

                is_exact = false;
                svc.event_target().emit(ev, args, &mut flow);

                if !flow.bubble() {
                    break;
                }
            }
        }
    }
}
