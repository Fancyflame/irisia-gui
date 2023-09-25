use std::cell::Cell;

use irisia_backend::StaticWindowEvent;

use crate::{
    application::content::GlobalContent,
    event::EventDispatcher,
    primitive::{Pixel, Point},
};

use super::{GlobalEventMgr, PointerState};

pub struct NewPointerEvent<'a> {
    pub(crate) event: StaticWindowEvent,
    pub(crate) gem: &'a mut GlobalEventMgr,
    pub(crate) global_content: &'a GlobalContent,
    pub(crate) new_position: Option<Point>,
    pub(crate) cursor_delta: Option<(Pixel, Pixel)>,
    new_focused: Cell<NewFocused>,
    pub(crate) new_pointer_state: PointerState,
    pub(crate) pointer_state_change: PointerStateChange,
}

enum NewFocused {
    Unchanged,
    ChangeTo(EventDispatcher),
    Blur,
}

#[derive(Clone, Copy, Debug)]
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
        gc: &'a GlobalContent,
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
            global_content: gc,
        }
    }

    pub(crate) fn focus_on(&self, ed: Option<EventDispatcher>) {
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
            NewFocused::ChangeTo(ed) => self.global_content.focusing.focus(ed),
            NewFocused::Blur => self.global_content.focusing.blur(),
        }

        self.gem.pointer_state = self.new_pointer_state;
    }
}
