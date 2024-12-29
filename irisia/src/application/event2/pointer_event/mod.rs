use crate::primitive::{Point, Region};
pub(crate) use state::PointerState;

mod state;

#[derive(Clone, Copy)]
pub enum PointerEvent {
    PointerDown {
        is_current: bool,
        position: Point,
    },

    PointerUp {
        is_current: bool,
        position: Point,
    },

    PointerMove {
        is_current: bool,
        delta: Point,
        position: Point,
    },

    PointerEntered,
    PointerOut,
    PointerOver,
    PointerLeft,
}

pub struct PointerStateDelta {
    pub(crate) prev: PointerState,
    pub(crate) next: PointerState,
    pub(crate) cursor_may_over: bool,
}

impl PointerStateDelta {
    pub fn get_event(
        &mut self,
        draw_region: Region,
        prev_cursor_directly_over: &mut bool,
    ) -> impl Iterator<Item = PointerEvent> {
        let (iter, cursor_over) = self.prev.compare(
            &self.next,
            draw_region,
            *prev_cursor_directly_over,
            self.cursor_may_over,
        );

        *prev_cursor_directly_over = cursor_over;
        if cursor_over {
            self.cursor_may_over = false;
        }
        iter
    }
}
