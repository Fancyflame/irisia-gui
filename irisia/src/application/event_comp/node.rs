use crate::{
    event::{
        standard::{
            PointerDown, PointerEntered, PointerLeft, PointerMove, PointerOut, PointerOver,
            PointerUp,
        },
        EventDispatcher,
    },
    primitive::{Pixel, Point, Region},
};

use super::{global::new_event::PointerStateChange, NewPointerEvent};

pub(crate) struct NodeEventMgr {
    ed: EventDispatcher,
    current_state: State,
}

#[derive(Clone, Copy, Debug)]
enum State {
    Untracked,
    LogicallyEnter,
    PhysicallyEnter,
}

impl NodeEventMgr {
    pub fn new(ed: EventDispatcher) -> Self {
        Self {
            ed,
            current_state: State::Untracked,
        }
    }

    pub fn update_and_emit(
        &mut self,
        update: &NewPointerEvent,
        region: Option<Region>,
        logically_entered: bool,
    ) -> bool {
        let position = match (update.new_position, region) {
            (Some(p), Some(region)) if p.abs_ge(region.0) && p.abs_le(region.1) => {
                self.update_state(State::PhysicallyEnter);
                p
            }
            (Some(p), None) if logically_entered => {
                self.update_state(State::LogicallyEnter);
                p
            }
            _ => {
                self.update_state(State::Untracked);
                return false;
            }
        };

        self.emit_physical_pointer_event(
            update.pointer_state_change,
            position,
            update.cursor_delta,
            logically_entered,
        );

        self.ed.emit_sys(update.event.clone());

        if let (PointerStateChange::Press, false) = (update.pointer_state_change, logically_entered)
        {
            // TODO: the element may cannot be focused on, set `None` instead.
            update.focus_on(Some(self.ed.clone()));
        }

        true
    }

    fn update_state(&mut self, new_state: State) {
        use State::*;

        let ed = &self.ed;
        let old_state = std::mem::replace(&mut self.current_state, new_state);

        match (old_state, self.current_state) {
            (Untracked, LogicallyEnter) => {
                ed.emit_sys(PointerEntered);
            }
            (LogicallyEnter, PhysicallyEnter) => {
                ed.emit_sys(PointerOver);
            }
            (Untracked, PhysicallyEnter) => {
                ed.emit_sys(PointerEntered);
                ed.emit_sys(PointerOver);
            }
            (PhysicallyEnter, Untracked) => {
                ed.emit_sys(PointerLeft);
                ed.emit_sys(PointerOut);
            }
            (PhysicallyEnter, LogicallyEnter) => {
                ed.emit_sys(PointerLeft);
            }
            (LogicallyEnter, Untracked) => {
                ed.emit_sys(PointerOut);
            }
            (Untracked, Untracked)
            | (LogicallyEnter, LogicallyEnter)
            | (PhysicallyEnter, PhysicallyEnter) => {}
        }
    }

    fn emit_physical_pointer_event(
        &self,
        psc: PointerStateChange,
        position: Point,
        delta: Option<(Pixel, Pixel)>,
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
                delta: delta.unwrap_or_else(|| {
                    if cfg!(debug_assertions) {
                        unreachable!("delta distance must be exist")
                    } else {
                        Default::default()
                    }
                }),
                position,
            }),
            PointerStateChange::Release => self.ed.emit_sys(PointerUp {
                is_current: logically_entered,
                position,
            }),
        }
    }
}
