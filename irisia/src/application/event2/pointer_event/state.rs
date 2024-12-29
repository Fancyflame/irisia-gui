use irisia_backend::winit::event::{ElementState, MouseButton, Touch, TouchPhase, WindowEvent};
use smallvec::SmallVec;

use crate::primitive::{Point, Region};

use super::PointerEvent;

#[derive(Clone, Copy)]
pub(crate) struct PointerState {
    cursor_position: Option<Point>,
    pressing: bool,
}

impl PointerState {
    pub fn new() -> Self {
        Self {
            cursor_position: None,
            pressing: false,
        }
    }

    pub fn next(&self, event: &WindowEvent) -> Option<Self> {
        let mut new_pressing = self.pressing;

        let mut new_position: Option<Point> = match &event {
            WindowEvent::Touch(touch) => Some(touch.location.into()),
            _ => self.cursor_position,
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
                new_pressing = true;
            }

            WindowEvent::CursorMoved { position, .. } => {
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
                new_pressing = false;
            }

            WindowEvent::CursorLeft { .. }
            | WindowEvent::Touch(Touch {
                phase: TouchPhase::Cancelled,
                ..
            }) => {
                new_position = None;
                new_pressing = false;
            }

            _ => return None,
        }

        Some(PointerState {
            cursor_position: new_position,
            pressing: new_pressing,
        })
    }

    fn pointer_inside(&self, region: Region) -> bool {
        match self.cursor_position {
            Some(pos) => region.contains_point(pos),
            None => false,
        }
    }

    pub(super) fn compare(
        &self,
        next: &Self,
        draw_region: Region,
        prev_cursor_over: bool,
        next_may_cursor_over: bool,
    ) -> (impl Iterator<Item = PointerEvent> + 'static, bool) {
        let mut events: SmallVec<[PointerEvent; 4]> = SmallVec::new();
        let inside_draw_region = (
            self.pointer_inside(draw_region),
            next.pointer_inside(draw_region),
        );

        let cursor_over = (
            prev_cursor_over,
            inside_draw_region.1 && next_may_cursor_over,
        );

        match inside_draw_region {
            (false, true) => events.push(PointerEvent::PointerEntered),
            (true, false) => events.push(PointerEvent::PointerOut),
            _ => {}
        }

        match cursor_over {
            (false, true) => events.push(PointerEvent::PointerOver),
            (true, false) => events.push(PointerEvent::PointerLeft),
            _ => {}
        }

        // if the pointer is outside, then skip following
        if !inside_draw_region.1 {
            return (events.into_iter(), false);
        }

        let is_current = cursor_over.1;
        let new_position = next.cursor_position.unwrap_or_default();

        if let (Some(pos1), Some(pos2)) = (self.cursor_position, next.cursor_position) {
            let delta = pos2 - pos1;
            if delta.0 != 0.0 && delta.1 != 0.0 {
                events.push(PointerEvent::PointerMove {
                    is_current,
                    delta,
                    position: new_position,
                });
            }
        }

        match (self.pressing, next.pressing) {
            (false, true) => events.push(PointerEvent::PointerDown {
                is_current,
                position: new_position,
            }),
            (true, false) => events.push(PointerEvent::PointerUp {
                is_current,
                position: new_position,
            }),
            _ => {}
        }

        (events.into_iter(), true)
    }
}
