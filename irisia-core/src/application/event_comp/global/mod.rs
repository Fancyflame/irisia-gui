use irisia_backend::{
    winit::event::{ElementState, MouseButton, Touch, TouchPhase},
    StaticWindowEvent,
};

use crate::{
    event::{
        standard::{PointerDown, PointerEntered, PointerMove, PointerOut, PointerUp},
        EventDispatcher,
    },
    primitive::{Pixel, Point},
};

use self::{
    focusing::Focusing,
    new_event::{NewPointerEvent, PointerStateChange},
};

pub(crate) mod focusing;
pub(crate) mod new_event;

pub(crate) struct GlobalEventMgr {
    global_ed: EventDispatcher,
    last_cursor_position: Option<Point>,
    focusing: Focusing,
    pointer_state: PointerState,
}

#[derive(Clone, Copy)]
pub(crate) enum PointerState {
    Pressing,
    Release,
    OutOfViewport,
}

impl GlobalEventMgr {
    pub fn new(global_ed: EventDispatcher) -> Self {
        GlobalEventMgr {
            global_ed,
            last_cursor_position: None,
            focusing: Focusing::new(),
            pointer_state: PointerState::OutOfViewport,
        }
    }

    pub fn global_ed(&self) -> &EventDispatcher {
        &self.global_ed
    }

    pub fn focusing(&self) -> &Focusing {
        &self.focusing
    }

    #[must_use]
    pub fn emit_event(&mut self, event: StaticWindowEvent) -> Option<NewPointerEvent> {
        match cursor_behavior(&event, self.pointer_state, self.last_cursor_position) {
            Some((new_position, new_pointer_state)) => {
                let npe = NewPointerEvent::new(event, self, new_position, new_pointer_state);
                npe.gem.emit_physical_pointer_event(
                    new_position,
                    npe.cursor_delta,
                    npe.pointer_state_change,
                );
                Some(npe)
            }
            None => {
                self.global_ed.emit_sys(event);
                None
            }
        }
    }

    fn emit_physical_pointer_event(
        &self,
        position: Option<Point>,
        delta: Option<(Pixel, Pixel)>,
        new_pointer_state: PointerStateChange,
    ) {
        let ed = &self.global_ed;

        match (new_pointer_state, position) {
            (PointerStateChange::EnterViewport, None) => ed.emit_sys(PointerEntered),
            (PointerStateChange::Press, Some(position)) => ed.emit_sys(PointerDown {
                is_current: false,
                position,
            }),
            (PointerStateChange::Unchange, Some(position)) => ed.emit_sys(PointerMove {
                is_current: false,
                delta: delta.unwrap(),
                position,
            }),
            (PointerStateChange::Release, Some(position)) => ed.emit_sys(PointerUp {
                is_current: false,
                position,
            }),
            (PointerStateChange::LeaveViewport, None) => ed.emit_sys(PointerOut),
            _ => unreachable!("unexpected new-pointer-state and optioned position combination"),
        }
    }
}

fn cursor_behavior(
    event: &StaticWindowEvent,
    old_state: PointerState,
    old_position: Option<Point>,
) -> Option<(Option<Point>, PointerState)> {
    let mut new_pointer_state = old_state;
    let mut new_position: Option<Point> = match &event {
        StaticWindowEvent::Touch(touch) => Some(touch.location.into()),
        _ => old_position,
    };

    match event {
        StaticWindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        }
        | StaticWindowEvent::Touch(Touch {
            phase: TouchPhase::Started,
            ..
        }) => {
            new_pointer_state = PointerState::Pressing;
        }

        StaticWindowEvent::CursorMoved { position, .. } => {
            new_position = Some(Point::from(*position))
        }

        StaticWindowEvent::Touch(Touch {
            phase: TouchPhase::Moved,
            ..
        }) => {}

        StaticWindowEvent::MouseInput {
            state: ElementState::Released,
            button: MouseButton::Left,
            ..
        }
        | StaticWindowEvent::Touch(Touch {
            phase: TouchPhase::Ended,
            ..
        }) => {
            new_pointer_state = PointerState::Release;
        }

        StaticWindowEvent::CursorLeft { .. }
        | StaticWindowEvent::Touch(Touch {
            phase: TouchPhase::Cancelled,
            ..
        }) => {
            new_pointer_state = PointerState::OutOfViewport;
        }

        _ => return None,
    }

    Some((new_position, new_pointer_state))
}
