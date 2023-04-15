use std::{collections::HashSet, hash::Hash};

use irisia_backend::{
    winit::{dpi::PhysicalPosition, event::DeviceId},
    StaticWindowEvent,
};

use crate::{
    event::{event_dispatcher::WeakEventDispatcher, standard::PointerLeft, EventDispatcher},
    primary::Point,
};

use super::ElemTable;

struct Item {
    ed: EventDispatcher,
    alive: bool,
}

#[derive(Default)]
pub struct CursorRecorder {
    cursor_pos: Option<Point>,
    entered: Vec<Item>,
    over: Vec<Item>,
}

impl CursorRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    fn clean(&mut self) {
        self.entered.retain(|Item { ed, alive }| {
            if !alive {
                ed.emit_sys(PointerLeft);
            }
            *alive
        });
        self.over
    }

    pub fn update(&mut self, table: &ElemTable, event: &StaticWindowEvent) {
        self.cursor_pos = match event {
            StaticWindowEvent::CursorLeft { .. } => None,
            StaticWindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => Some(Point(*x as _, *y as _)),
            _ => return,
        };
    }
}
