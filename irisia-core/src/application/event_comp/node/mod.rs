use crate::{
    event::{
        standard::{
            PointerDown, PointerEntered, PointerLeft, PointerMove, PointerOut, PointerOver,
            PointerUp,
        },
        EventDispatcher,
    },
    primitive::{Point, Region},
};

use super::global::new_event::{NewEvent, PointerStateChange};

pub(crate) struct NodeEventMgr {
    ed: EventDispatcher,
    current_state: State,
}

#[derive(Clone, Copy)]
enum State {
    Outside,
    LogicallyEnter,
    PhysicallyEnter,
}

impl NodeEventMgr {
    pub fn update_and_emit(&mut self, update: &NewEvent, region: Region, logically_entered: bool) {
        let pe = match update {
            NewEvent::Common(ev) => {
                self.ed.emit_sys(ev.clone());
                return;
            }
            NewEvent::PointerEvent(pe) => pe,
        };

        let position = match pe.new_position {
            Some(p) if p.abs_ge(region.0) && p.abs_le(region.1) => {
                self.update_state(State::PhysicallyEnter);
                p
            }
            Some(p) if logically_entered => {
                self.update_state(State::LogicallyEnter);
                p
            }
            _ => {
                self.update_state(State::Outside);
                return;
            }
        };

        self.emit_physical_pointer_event(pe.pointer_state_change, position, logically_entered);
    }

    fn update_state(&mut self, new_state: State) {
        use State::*;

        let ed = &self.ed;
        let old_state = std::mem::replace(&mut self.current_state, new_state);

        match (old_state, self.current_state) {
            (Outside, LogicallyEnter) => {
                ed.emit_sys(PointerEntered);
            }
            (LogicallyEnter, PhysicallyEnter) => {
                ed.emit_sys(PointerOver);
            }
            (Outside, PhysicallyEnter) => {
                ed.emit_sys(PointerEntered);
                ed.emit_sys(PointerOver);
            }
            (PhysicallyEnter, Outside) => {
                ed.emit_sys(PointerLeft);
                ed.emit_sys(PointerOut);
            }
            (PhysicallyEnter, LogicallyEnter) => {
                ed.emit_sys(PointerLeft);
            }
            (LogicallyEnter, Outside) => {
                ed.emit_sys(PointerOut);
            }
            (Outside, Outside)
            | (LogicallyEnter, LogicallyEnter)
            | (PhysicallyEnter, PhysicallyEnter) => {}
        }
    }

    fn emit_physical_pointer_event(
        &self,
        psc: PointerStateChange,
        position: Point,
        logically_entered: bool,
    ) {
        match psc {
            PointerStateChange::EnterViewport { .. } | PointerStateChange::LeaveViewport => {}
            PointerStateChange::Press => self.ed.emit_sys(PointerDown {
                is_current: logically_entered,
                position,
            }),
            PointerStateChange::Unchange => self.ed.emit_sys(PointerMove {
                is_current: logically_entered,
                position,
            }),
            PointerStateChange::Release => self.ed.emit_sys(PointerUp {
                is_current: logically_entered,
                position,
            }),
        }
    }
}
