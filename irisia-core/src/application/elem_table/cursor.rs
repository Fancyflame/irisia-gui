use std::collections::{hash_map::Entry, HashMap};

use irisia_backend::{winit::dpi::PhysicalPosition, StaticWindowEvent};

use crate::{
    event::{
        standard::{PointerEntered, PointerLeft, PointerOut, PointerOver},
        EventDispatcher,
    },
    primary::Point,
};

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
    _ed: OutProtected,
}

#[derive(Default)]
pub struct CursorWatcher {
    cursor_pos: Option<Point>,
    entered: HashMap<usize, Alive>,
    over: Option<LeaveProtected>,
}

impl CursorWatcher {
    pub fn new() -> Self {
        Self::default()
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
        while let Some(item) = next {
            next = item.parent.map(|index| &registered[index]);
            let entry = self.entered.entry(item.event_dispatcher.as_ptr() as usize);
            match entry {
                Entry::Vacant(vacant) => {
                    item.event_dispatcher.emit_sys(PointerEntered);
                    vacant.insert(Alive {
                        alive: true,
                        _ed: OutProtected(item.event_dispatcher.clone()),
                    });
                }
                Entry::Occupied(mut occupied) => occupied.get_mut().alive = true,
            }
        }
        self.entered.retain(|_, v| v.alive);

        self.update_pointer_over(next.map(|x| &x.event_dispatcher));
    }

    pub(super) fn cursor_pos(&self) -> Option<Point> {
        self.cursor_pos
    }

    pub(super) fn update(&mut self, registered: &[super::Item], event: &StaticWindowEvent) {
        self.cursor_pos = match event {
            StaticWindowEvent::CursorLeft { .. } => None,
            StaticWindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => Some(Point(*x as _, *y as _)),
            _ => return,
        };

        self.update_chain_in_place(
            registered,
            self.cursor_pos
                .and_then(|point| super::cursor_on(registered, point)),
        );
    }
}
