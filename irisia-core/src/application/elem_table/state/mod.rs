use std::time::{Duration, Instant};

use irisia_backend::{
    winit::{
        dpi::PhysicalPosition,
        event::{ElementState, MouseButton, Touch, TouchPhase},
    },
    StaticWindowEvent,
};

//use self::__pointer::PointerState;

mod pointer;

pub struct StateRecorder {
    pointer_state: PointerState,
}

impl StateRecorder {
    pub fn new() -> Self {
        StateRecorder {
            pointer_state: PointerState::new(),
        }
    }

    pub fn update(&mut self, event: &StaticWindowEvent) {
        let pe = self.pointer_state.update(event);
        todo!()
    }
}
