use std::collections::{hash_map::Entry, HashMap};

use irisia_backend::StaticWindowEvent;

use crate::{
    event::{
        standard::{PointerEntered, PointerLeft, PointerOut, PointerOver},
        EventDispatcher,
    },
    primary::Point,
};

use self::pointer_event::Advanced;

mod pointer_event;

struct OutProtected(EventDispatcher);

impl Drop for OutProtected {
    fn drop(&mut self) {
        self.0.emit_sys(PointerOut)
    }
}

struct LeaveProtected(EventDispatcher);

impl Drop for LeaveProtected {
    fn drop(&mut self) {
        self.0.emit_sys(PointerLeft)
    }
}

struct Alive {
    alive: bool,
    ed: OutProtected,
}

#[derive(Default)]
pub struct CursorWatcher {
    cursor_pos: Option<Point>,
    advanced: Advanced,
    entered: HashMap<usize, Alive>,
    over: Option<LeaveProtected>,
}

impl CursorWatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn top_element(&self) -> Option<&EventDispatcher> {
        self.over.as_ref().map(|x| &x.0)
    }

    fn update_pointer_over(&mut self, new: Option<&EventDispatcher>) {
        match (&mut self.over, new) {
            (Some(old), Some(new)) if old.0.is_same(&new) => {}
            (place, new) => {
                *place = new.map(|ed| {
                    ed.emit_sys(PointerOver);
                    LeaveProtected(ed.clone())
                });
            }
        }
    }

    fn update_chain_in_place(&mut self, registered: &[super::Item], index: Option<usize>) {
        for alive in self.entered.values_mut() {
            alive.alive = false;
        }

        let mut next = index.map(|index| &registered[index]);
        self.update_pointer_over(next.map(|x| &x.event_dispatcher));

        while let Some(item) = next {
            next = item.parent.map(|index| &registered[index]);
            let entry = self.entered.entry(item.event_dispatcher.as_ptr() as usize);
            match entry {
                Entry::Vacant(vacant) => {
                    item.event_dispatcher.emit_sys(PointerEntered);
                    vacant.insert(Alive {
                        alive: true,
                        ed: OutProtected(item.event_dispatcher.clone()),
                    });
                }
                Entry::Occupied(mut occupied) => occupied.get_mut().alive = true,
            }
        }
        self.entered.retain(|_, v| v.alive);
    }

    // returns clicked element
    pub(super) fn update(&mut self, registered: &[super::Item], event: &StaticWindowEvent) -> bool {
        self.update_chain_in_place(
            registered,
            self.cursor_pos
                .and_then(|point| super::cursor_on(registered, point)),
        );
        self.update_advanced(event)
    }
}
