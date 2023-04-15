use std::time::{Duration, Instant};

use irisia_backend::{
    winit::{
        dpi::PhysicalPosition,
        event::{ElementState, MouseButton, Touch, TouchPhase},
    },
    StaticWindowEvent,
};

use crate::primary::Point;

pub struct PointerState {
    pos: Point,
    press: Option<PressRec>,
}

struct PressRec {
    start_time: Instant,
    start_pos: Point,
}

impl PointerState {
    pub fn new() -> Self {
        PointerState {
            pos: Point(0, 0),
            press: None,
        }
    }

    pub fn update(&mut self, event: &StaticWindowEvent) -> BasicState {
        const CLICK_LONGEST_INTERVAL: Duration = Duration::from_micros(300);
        match (event, self.press) {
            (
                StaticWindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                }
                | StaticWindowEvent::Touch(Touch {
                    phase: TouchPhase::Cancelled | TouchPhase::Ended,
                    ..
                }),
                None,
            ) => {
                let delta = BasicState::Down { pos: self.pos };
                self.press = Some(PressRec {
                    start_time: Instant::now(),
                    start_pos: self.pos,
                });
                delta
            }

            (
                StaticWindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Left,
                    ..
                }
                | StaticWindowEvent::Touch(Touch {
                    phase: TouchPhase::Started,
                    ..
                }),
                Some(PressRec {
                    start_time,
                    start_pos,
                }),
            ) => {
                let is_click = {
                    let diff = ((self.pos.0 as f32 - start_pos.0 as f32).powi(2)
                        + (self.pos.1 as f32 - start_pos.1 as f32).powi(2))
                    .sqrt();
                    start_time.elapsed() < CLICK_LONGEST_INTERVAL && diff < 50.0
                };

                let delta = BasicState::Up {
                    pos: self.pos,
                    is_click,
                };

                self.press = None;
                delta
            }

            (
                StaticWindowEvent::CursorMoved { position, .. }
                | StaticWindowEvent::Touch(Touch {
                    phase: TouchPhase::Moved,
                    location: position,
                    ..
                }),
                _,
            ) => {
                let PhysicalPosition { x, y } = *position;
                let new_point = Point(x as _, y as _);
                let delta_dist = (x as i32 - self.pos.0 as i32, y as i32 - self.pos.1 as i32);

                let delta = BasicState::Move {
                    pos: new_point,
                    delta: delta_dist,
                };

                self.pos = new_point;

                delta
            }

            _ => BasicState::None,
        }
    }
}

pub enum BasicState {
    Down { pos: Point },
    Up { pos: Point, is_click: bool },
    Move { pos: Point, delta: (i32, i32) },
    None,
}
