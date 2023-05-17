use std::sync::Arc;

use crate::{
    event::EventDispatcher,
    primitive::{Point, Region},
};

use tokio::sync::Mutex;

use focus::Focusing;
use irisia_backend::StaticWindowEvent;

use self::{cursor::CursorWatcher, focus::SharedFocusing};

mod cursor;
pub(crate) mod focus;

struct Item {
    event_dispatcher: EventDispatcher,
    interact_region: Option<Region>,
    parent: Option<usize>,
}

pub(crate) struct EventComponent {
    global: EventDispatcher,
    registered: Vec<Item>,
    builder_stack: Vec<usize>,
    focusing: SharedFocusing,
    cursor_watcher: CursorWatcher,
}

impl EventComponent {
    pub fn new(global: EventDispatcher) -> Self {
        EventComponent {
            global,
            registered: Vec::new(),
            builder_stack: Vec::new(),
            focusing: Arc::new(Mutex::new(Focusing::new())),
            cursor_watcher: CursorWatcher::new(),
        }
    }

    pub fn rebuild<F, Ret>(&mut self, f: F) -> Ret
    where
        F: FnOnce(Builder, &EventDispatcher, &SharedFocusing) -> Ret,
    {
        self.registered.clear();

        let mut focusing = match self.focusing.try_lock() {
            Ok(guard) => guard,
            Err(_) => self.focusing.blocking_lock(),
        };
        focusing.to_not_confirmed();

        let mut builder = Builder {
            items: &mut self.registered,
            focusing: &mut focusing,
            builder_stack: &mut self.builder_stack,
        };

        builder.push(self.global.clone());
        let ret = f(builder.downgrade_lifetime(), &self.global, &self.focusing);
        builder.finish();

        focusing.drop_not_confirmed();
        ret
    }

    pub fn emit_window_event(&mut self, event: StaticWindowEvent) {
        let clicked = self.cursor_watcher.update(&self.registered, &event);

        if clicked {
            match self.cursor_watcher.top_element() {
                Some(ed) => {
                    self.focusing.blocking_lock().focus_on(ed.clone());
                }
                None => {
                    self.focusing.blocking_lock().blur();
                }
            }
        }

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
    items: &'a mut Vec<Item>,
    focusing: &'a mut Focusing,
    builder_stack: &'a mut Vec<usize>,
}

impl Builder<'_> {
    pub fn downgrade_lifetime(&mut self) -> Builder {
        Builder {
            items: self.items,
            focusing: self.focusing,
            builder_stack: self.builder_stack,
        }
    }

    pub fn push(&mut self, event_dispatcher: EventDispatcher) -> usize {
        let index = self.items.len();

        self.focusing.try_confirm(&event_dispatcher);
        self.items.push(Item {
            event_dispatcher,
            interact_region: None,
            parent: self.builder_stack.last().copied(),
        });
        self.builder_stack.push(index);

        index
    }

    pub fn set_interact_region_for(&mut self, index: usize, r: Option<Region>) {
        self.items[index].interact_region = r;
    }

    pub fn finish(&mut self) {
        assert!(self.builder_stack.pop().is_some());
    }
}
