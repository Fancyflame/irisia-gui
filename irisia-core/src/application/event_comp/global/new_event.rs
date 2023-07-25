use std::cell::Cell;

use irisia_backend::StaticWindowEvent;

use crate::{
    event::EventDispatcher,
    primitive::{Pixel, Point},
};

use super::{GlobalEventMgr, PointerState};

pub(crate) enum NewEvent<'a> {
    PointerEvent(NewPointerEvent<'a>),
    Common(StaticWindowEvent),
}

pub(crate) struct NewPointerEvent<'a> {
    pub event: StaticWindowEvent,
    pub gem: &'a mut GlobalEventMgr,
    pub new_position: Option<Point>,
    pub cursor_delta: Option<(Pixel, Pixel)>,
    new_focused: Cell<NewFocused>,
    pub new_pointer_state: PointerState,
    pub pointer_state_change: PointerStateChange,
}

enum NewFocused {
    Unchanged,
    ChangeTo(EventDispatcher),
    Blur,
}

#[derive(Clone, Copy)]
pub(crate) enum PointerStateChange {
    Unchange,
    Press,
    Release,
    LeaveViewport,
    EnterViewport,
}

impl<'a> NewPointerEvent<'a> {
    pub(super) fn new(
        event: StaticWindowEvent,
        gem: &'a mut GlobalEventMgr,
        new_position: Option<Point>,
        new_pointer_state: PointerState,
    ) -> Self {
        let cursor_delta = gem
            .last_cursor_position
            .zip(new_position)
            .map(|(old, new)| (new.0 - old.0, new.1 - old.1));

        NewPointerEvent {
            event,
            new_position,
            cursor_delta,
            new_focused: Cell::new(NewFocused::Unchanged),
            new_pointer_state,
            pointer_state_change: PointerStateChange::difference_between(
                gem.pointer_state,
                new_pointer_state,
            ),
            gem,
        }
    }

    pub(crate) fn default_focus_on(&self, ed: Option<EventDispatcher>) {
        self.new_focused.set(match ed {
            Some(ed) => NewFocused::ChangeTo(ed),
            None => NewFocused::Blur,
        });
    }
}

impl PointerStateChange {
    fn difference_between(old: PointerState, new: PointerState) -> Self {
        use PointerState::*;

        match (old, new) {
            (Release, Pressing) => Self::Press,
            (Pressing, Release) => Self::Release,
            (OutOfViewport, Pressing | Release) => Self::EnterViewport,
            (Pressing | Release, OutOfViewport) => Self::LeaveViewport,
            (Release, Release) | (Pressing, Pressing) | (OutOfViewport, OutOfViewport) => {
                Self::Unchange
            }
        }
    }
}

impl Drop for NewPointerEvent<'_> {
    fn drop(&mut self) {
        self.gem.last_cursor_position = self.new_position;

        match self.new_focused.replace(NewFocused::Unchanged) {
            NewFocused::Unchanged => (),
            NewFocused::ChangeTo(ed) => self.gem.focusing.focus(ed),
            NewFocused::Blur => self.gem.focusing.blur(),
        }

        self.gem.pointer_state = self.new_pointer_state;
    }
}
