use std::time::{Duration, Instant};

use irisia_backend::{
    winit::{
        dpi::PhysicalPosition,
        event::{ElementState, MouseButton, Touch, TouchPhase},
    },
    StaticWindowEvent,
};

use crate::{
    event::{
        standard::{Click, PointerDown, PointerMove, PointerUp},
        EventDispatcher,
    },
    primary::Point,
    Event,
};

use super::CursorWatcher;

#[derive(Default)]
pub(super) struct Advanced {
    state: Option<Pressed>,
}

struct Pressed {
    ed: Option<EventDispatcher>,
    time: Instant,
    position: Option<Point>,
}

fn emit_event<F, E>(this: &CursorWatcher, mut f: F)
where
    F: FnMut(bool) -> E,
    E: Event,
{
    let current = match &this.over {
        Some(pos) => &pos.0,
        None => return,
    };

    for alive in this.entered.values() {
        alive.ed.0.emit_sys(f(current.is_same(&alive.ed.0)));
    }
}

impl super::CursorWatcher {
    // returns whether click behavior triggered.
    pub(super) fn update_advanced(&mut self, event: &StaticWindowEvent) -> bool {
        if let StaticWindowEvent::Touch(Touch { location, .. }) = event {
            self.cursor_pos = Some({
                let PhysicalPosition { x, y } = *location;
                Point(x as _, y as _)
            });
        }

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
                let is_leading = match &mut self.advanced.state {
                    Some(_) => false,
                    None => {
                        self.set_pressed();
                        true
                    }
                };

                emit_event(&self, |is_first| PointerDown {
                    is_leading,
                    position: self.cursor_pos.unwrap_or_default(),
                    is_current: is_first,
                });
            }

            StaticWindowEvent::CursorMoved { position, .. }
            | StaticWindowEvent::Touch(Touch {
                phase: TouchPhase::Moved,
                location: position,
                ..
            }) => {
                let new_pos = {
                    let PhysicalPosition { x, y } = *position;
                    Point(x as _, y as _)
                };

                self.cursor_pos = Some(new_pos);

                emit_event(self, |is_first| PointerMove {
                    is_current: is_first,
                    position: new_pos,
                });
            }

            StaticWindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            }
            | StaticWindowEvent::Touch(Touch {
                phase: TouchPhase::Cancelled | TouchPhase::Ended,
                ..
            }) => {
                emit_event(self, |is_first| PointerUp {
                    is_current: is_first,
                    position: self.cursor_pos.unwrap_or_default(),
                });
                return self.handle_click();
            }

            StaticWindowEvent::CursorLeft { .. } => {
                self.cursor_pos = None;
            }

            _ => {}
        }

        false
    }

    fn set_pressed(&mut self) {
        self.advanced.state = Some(Pressed {
            ed: self.over.as_ref().map(|protected| protected.0.clone()),
            time: Instant::now(),
            position: self.cursor_pos,
        });
    }

    fn handle_click(&mut self) -> bool {
        const CLICK_LONGEST_INTERVAL: Duration = Duration::from_millis(300);

        let Some(press_info) = self
            .advanced
            .state
            .take()
        else {
            if cfg!(debug_assertions){
                inner_error!("pointer down info is missing");
            } else {
                return false;
            }
        };

        let (Some(pos_now), Some(pos_prv)) = (self.cursor_pos, press_info.position)
        else {
            return false;
        };

        let distance = ((pos_now.0 as f32 - pos_prv.0 as f32).powi(2)
            + (pos_now.1 as f32 - pos_prv.1 as f32).powi(2))
        .sqrt();

        if press_info.time.elapsed() > CLICK_LONGEST_INTERVAL || distance > 40.0 {
            return false;
        }

        match (&self.over, &press_info.ed) {
            (Some(x), Some(ed)) if ed.is_same(&x.0) => {}
            _ => return false,
        }

        emit_event(&self, |is_first| Click {
            is_current: is_first,
        });
        true
    }
}
