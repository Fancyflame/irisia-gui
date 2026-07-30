use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use irisia_unsafe::any_queue::{AnyBox, AnyQueue};

use crate::{System, utils::ReasonCell};

type RcSystem = Rc<ReasonCell<dyn StoredSystem>>;

trait StoredSystem: System + Any {
    fn receive_event(&mut self, event: &AnyBox);
}

#[derive(Default)]
pub(super) struct SystemStock {
    map: RefCell<HashMap<TypeId, RcSystem>>,
    event_queue: RefCell<AnyQueue>,
    event_exec_buffer: RefCell<EventExecuteBuffer>,
}

#[derive(Default)]
struct EventExecuteBuffer {
    any_box: AnyBox,
    system_queue: Vec<RcSystem>,
}

impl SystemStock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit_event<E: 'static>(&self, event: E) {
        self.event_queue.borrow_mut().push_back(event);
    }

    pub fn execute_event(&self) {
        let mut event_exec_buffer_refm = self
            .event_exec_buffer
            .try_borrow_mut()
            .expect("cannot execute event inside execution");

        let EventExecuteBuffer {
            any_box,
            system_queue,
        } = &mut *event_exec_buffer_refm;

        while self
            .event_queue
            .try_borrow_mut()
            .expect("event queue should never failed on calling borrow_mut")
            .pop_front(any_box)
        {
            system_queue.clear();
            system_queue.extend(self.map.borrow().values().cloned());

            for sys in system_queue.drain(..) {
                let mut sys = sys.borrow_mut(
                    "cannot borrow system from global while the system is processing event",
                );
                sys.receive_event(any_box);
            }

            any_box.clear_value();
        }
    }
}
