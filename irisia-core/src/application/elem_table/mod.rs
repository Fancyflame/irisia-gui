use crate::{
    event::EventDispatcher,
    primary::{Point, Region},
    Event,
};

use focus::Focusing;
use irisia_backend::StaticWindowEvent;

use self::cursor::CursorWatcher;

mod advanced;
mod cursor;
mod focus;

struct Item {
    event_dispatcher: EventDispatcher,
    interact_region: Option<Region>,
    parent: Option<usize>,
}

pub(crate) struct ElemTable {
    global: EventDispatcher,
    registered: Vec<Item>,
    builder_stack: Vec<usize>,
    focusing: Focusing,
    cursor_watcher: CursorWatcher,
}

impl ElemTable {
    pub fn new(global: EventDispatcher) -> Self {
        ElemTable {
            global,
            registered: Vec::new(),
            builder_stack: Vec::new(),
            focusing: Focusing::new(),
            cursor_watcher: CursorWatcher::new(),
        }
    }

    pub fn builder(&mut self) -> (Builder, &EventDispatcher) {
        self.registered.clear();
        self.focusing.to_not_confirmed();
        (
            Builder {
                is_root: true,
                items: &mut self.registered,
                focusing: &mut self.focusing,
                builder_stack: &mut self.builder_stack,
            },
            &self.global,
        )
    }

    pub fn emit_window_event(&mut self, event: StaticWindowEvent) {
        self.cursor_watcher.update(&self.registered, &event);

        let mut selected = self
            .cursor_watcher
            .cursor_pos()
            .and_then(|point| cursor_on(&self.registered, point));
        while let Some(index) = selected {
            let item = &self.registered[index];
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

fn cursor_on(registered: &[Item], point: Point) -> Option<usize> {
    let mut selected = None;

    for (index, item) in registered.iter().enumerate().rev() {
        if let Some(re) = item.interact_region {
            if point.abs_ge(re.0) && point.abs_le(re.1) {
                selected = Some(index);
                break;
            }
        }
    }

    selected
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
            parent: self.builder_stack.last().copied(),
        });
        self.builder_stack.push(index);

        index
    }

    pub fn set_interact_region_for(&mut self, index: usize, r: Region) {
        self.items[index].interact_region = Some(r);
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
