use irisia_backend::winit::event::{ElementState, MouseButton, Touch, TouchPhase, WindowEvent};

use crate::{
    application::content::GlobalContent,
    event::{
        standard::{
            CloseRequested, PointerDown, PointerEntered, PointerMove, PointerOut, PointerUp,
        },
        EventDispatcher,
    },
    primitive::{Pixel, Point},
};

use self::new_event::{IncomingPointerEvent, PointerStateChange};

pub(crate) mod focusing;
pub(crate) mod new_event;

pub(crate) struct GlobalEventMgr {
    last_cursor_position: Option<Point>,
    pointer_state: PointerState,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PointerState {
    Pressing,
    Release,
    OutOfViewport,
}

impl GlobalEventMgr {
    pub fn new() -> Self {
        GlobalEventMgr {
            last_cursor_position: None,
            pointer_state: PointerState::OutOfViewport,
        }
    }

    #[must_use]
    pub fn emit_event<'a>(
        &'a mut self,
        event: WindowEvent,
        gc: &'a GlobalContent,
    ) -> Option<IncomingPointerEvent<'a>> {
        match cursor_behavior(&event, self.pointer_state, self.last_cursor_position) {
            Some((new_position, new_pointer_state)) => {
                let ipe =
                    IncomingPointerEvent::new(event, self, gc, new_position, new_pointer_state);
                emit_physical_pointer_event(
                    &gc.global_ed,
                    new_position,
                    ipe.cursor_delta,
                    ipe.pointer_state_change,
                );
                Some(ipe)
            }
            None => {
                match &event {
                    WindowEvent::CloseRequested => {
                        gc.global_ed.emit_trusted(CloseRequested(gc.close_handle))
                    }
                    _ => {}
                }

                gc.global_ed.emit_trusted(event);
                None
            }
        }
    }
}

fn emit_physical_pointer_event(
    ed: &EventDispatcher,
    position: Option<Point>,
    delta: Option<(Pixel, Pixel)>,
    new_pointer_state: PointerStateChange,
) {
    match (new_pointer_state, position) {
        (PointerStateChange::EnterViewport, _) => ed.emit_trusted(PointerEntered),
        (PointerStateChange::Press, Some(position)) => ed.emit_trusted(PointerDown {
            is_current: false,
            position,
        }),
        (PointerStateChange::Unchange, Some(position)) => ed.emit_trusted(PointerMove {
            is_current: false,
            delta: delta.unwrap_or_default(),
            position,
        }),
        (PointerStateChange::Release, Some(position)) => ed.emit_trusted(PointerUp {
            is_current: false,
            position,
        }),
        (PointerStateChange::LeaveViewport, None) => ed.emit_trusted(PointerOut),
        _ => {
            unreachable!("unexpected new-pointer-state and optioned position combination")
        }
    }
}

fn cursor_behavior(
    event: &WindowEvent,
    old_state: PointerState,
    old_position: Option<Point>,
) -> Option<(Option<Point>, PointerState)> {
    let mut new_pointer_state = old_state;

    let mut new_position: Option<Point> = match &event {
        WindowEvent::Touch(touch) => Some(touch.location.into()),
        _ => old_position,
    };

    match event {
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        }
        | WindowEvent::Touch(Touch {
            phase: TouchPhase::Started,
            ..
        }) => {
            new_pointer_state = PointerState::Pressing;
        }

        WindowEvent::CursorMoved { position, .. } => {
            if let PointerState::OutOfViewport = new_pointer_state {
                new_pointer_state = PointerState::Release;
            }
            new_position = Some(Point::from(*position))
        }

        WindowEvent::Touch(Touch {
            phase: TouchPhase::Moved,
            ..
        }) => {}

        WindowEvent::MouseInput {
            state: ElementState::Released,
            button: MouseButton::Left,
            ..
        }
        | WindowEvent::Touch(Touch {
            phase: TouchPhase::Ended,
            ..
        }) => {
            new_pointer_state = PointerState::Release;
        }

        WindowEvent::CursorLeft { .. }
        | WindowEvent::Touch(Touch {
            phase: TouchPhase::Cancelled,
            ..
        }) => {
            new_position.take();
            new_pointer_state = PointerState::OutOfViewport;
        }

        _ => return None,
    }

    Some((new_position, new_pointer_state))
}
