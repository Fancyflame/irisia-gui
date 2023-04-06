use crate::{
    event::EventDispatcher,
    primary::{Point, Region},
    Event,
};

struct Item {
    event_dispatcher: EventDispatcher,
    interact_region: Option<Region>,
    parent: Option<usize>,
}

pub(crate) struct ElemTable {
    cursor_position: Option<Point>,
    emitters: Vec<Item>,
    builder_stack: Vec<usize>,
}

impl ElemTable {
    pub fn new() -> Self {
        ElemTable {
            cursor_position: None,
            emitters: Vec::new(),
            builder_stack: Vec::new(),
        }
    }

    pub fn builder(&mut self) -> Builder {
        self.emitters.clear();
        Builder {
            items: &mut self.emitters,
            builder_stack: &mut self.builder_stack,
        }
    }

    pub fn update_cursor(&mut self, position: Option<Point>) {
        self.cursor_position = position;
    }

    pub fn emit_sys<E>(&self, event: E)
    where
        E: Event,
    {
        let Some(point) = self.cursor_position else {
            return;
        };

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

        while let Some(index) = selected {
            let item = &self.emitters[index];
            item.event_dispatcher.emit_sys(event.clone());
            selected = item.parent;
        }
    }
}

pub(crate) struct Builder<'a> {
    items: &'a mut Vec<Item>,
    builder_stack: &'a mut Vec<usize>,
}

impl Builder<'_> {
    pub fn downgrade_lifetime(&mut self) -> Builder {
        Builder {
            items: self.items,
            builder_stack: self.builder_stack,
        }
    }

    pub fn push(&mut self, event_dispatcher: EventDispatcher) -> usize {
        self.items.push(Item {
            event_dispatcher,
            interact_region: None,
            parent: self.builder_stack.last().copied(),
        });

        let index = self.items.len() - 1;
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
