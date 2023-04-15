use crate::{
    event::EventDispatcher,
    primary::{Point, Region},
    Event,
};

use focus::Focusing;
use irisia_backend::StaticWindowEvent;

//use self::state::{InputState, StateRecorder};

mod cursor;
mod focus;
//mod state;

struct Item {
    event_dispatcher: EventDispatcher,
    interact_region: Option<Region>,
    parent: Option<usize>,
}

pub(crate) struct ElemTable {
    global: EventDispatcher,
    focusing: Focusing,
    emitters: Vec<Item>,
    builder_stack: Vec<usize>,
    state_recorder: StateRecorder,
}

impl ElemTable {
    pub fn new(global: EventDispatcher) -> Self {
        ElemTable {
            global,
            focusing: Focusing::new(),
            emitters: Vec::new(),
            builder_stack: Vec::new(),
            state_recorder: StateRecorder::new(),
        }
    }

    pub fn builder(&mut self) -> (Builder, &EventDispatcher) {
        self.emitters.clear();
        self.focusing.to_not_confirmed();
        (
            Builder {
                is_root: true,
                items: &mut self.emitters,
                focusing: &mut self.focusing,
                builder_stack: &mut self.builder_stack,
            },
            &self.global,
        )
    }

    fn cursor_on(&self, point: Point) -> Option<usize> {
        let mut selected = None;

        for (
            index,
            Item {
                interact_region, ..
            },
        ) in self.emitters.iter().enumerate().rev()
        {
            if let Some(re) = interact_region {
                if point.abs_ge(re.0) && point.abs_le(re.1) {
                    selected = Some(index);
                    break;
                }
            }
        }

        selected
    }

    pub fn emit_window_event(&mut self, event: StaticWindowEvent) {
        let input_state = self.state_recorder.update(&event);

        if let InputState {
            clicked: true,
            position: Some(pos),
        } = input_state
        {
            match self.cursor_on(pos) {
                Some(index) => self
                    .focusing
                    .focus_on(self.emitters[index].event_dispatcher.clone(), index),
                None => self.focusing.blur(),
            }
        }

        let mut selected = self.focusing.get_focused();
        while let Some(index) = selected {
            let item = &self.emitters[index];
            item.event_dispatcher.emit_sys(event.clone());
            selected = item.parent;
        }
        self.emit_sys(event);
    }

    pub fn emit_sys<E>(&self, event: E)
    where
        E: Event,
    {
        self.global.emit_sys(event);
    }
}

pub(crate) struct Builder<'a> {
    is_root: bool,
    items: &'a mut Vec<Item>,
    focusing: &'a mut Focusing,
    builder_stack: &'a mut Vec<usize>,
}

impl Builder<'_> {
    pub fn downgrade_lifetime(&mut self) -> Builder {
        Builder {
            is_root: false,
            items: self.items,
            focusing: self.focusing,
            builder_stack: self.builder_stack,
        }
    }

    pub fn push(&mut self, event_dispatcher: EventDispatcher) -> usize {
        let index = self.items.len();

        self.focusing.try_confirm(&event_dispatcher, index);
        self.items.push(Item {
            event_dispatcher,
            interact_region: None,
            can_focus_on: false,
            parent: self.builder_stack.last().copied(),
        });
        self.builder_stack.push(index);

        index
    }

    pub fn set_interact_region_for(&mut self, index: usize, r: Region) {
        self.items[index].interact_region = Some(r);
    }

    pub fn set_can_recv_focus_for(&mut self, index: usize) {
        self.items[index].can_focus_on = true;
    }

    pub fn finish(&mut self) {
        assert!(self.builder_stack.pop().is_some());
    }
}

impl Drop for Builder<'_> {
    fn drop(&mut self) {
        if self.is_root {
            self.focusing.drop_not_confirmed();
        }
    }
}
