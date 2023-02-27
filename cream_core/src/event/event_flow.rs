use crate::primary::{Point, Region};

use super::{event_state::wrap::WrappedEvents, Event};

#[derive(Debug, Clone)]
pub struct EventFlow {
    pub(crate) bubble: bool,
    pub(crate) is_current: bool,
}

impl EventFlow {
    pub fn bubble(&self) -> bool {
        self.bubble
    }

    pub fn is_current(&self) -> bool {
        self.is_current
    }

    pub fn cancel_bubble(&mut self) {
        self.bubble = false;
    }

    pub fn call_multiple<'a, E, I>(iter: I, event: &E, point: Option<Point>)
    where
        E: Event,
        I: Iterator<Item = &'a (WrappedEvents, Region)>,
    {
        let mut flow = EventFlow {
            bubble: true,
            is_current: true,
        };

        for (cb, region) in iter {
            if let Some(p) = point {
                if !(region.0.abs_lt(p) && region.1.abs_gt(p)) {
                    continue;
                }
            }

            cb.emit(event, &mut flow);

            if !flow.bubble {
                break;
            }

            flow.is_current = false;
        }
    }
}
