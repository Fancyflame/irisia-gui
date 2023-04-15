use crate::event::{
    standard::{Blured, Focused},
    EventDispatcher,
};

use self::protected::Protected;

mod protected {
    use super::*;

    pub struct Protected(EventDispatcher);

    impl Protected {
        pub fn new(ed: EventDispatcher) -> Self {
            ed.emit_sys(Focused);
            Protected(ed)
        }

        pub fn get(&self) -> &EventDispatcher {
            &self.0
        }
    }

    impl Drop for Protected {
        fn drop(&mut self) {
            self.0.emit_sys(Blured);
        }
    }
}

pub struct Focusing(Inner);

enum Inner {
    NotConfirmed(Protected),
    Confirmed { protected: Protected, index: usize },
    None,
}

impl Focusing {
    pub fn new() -> Self {
        Focusing(Inner::None)
    }

    pub fn get_focused(&self) -> Option<usize> {
        match &self.0 {
            Inner::Confirmed { index, .. } => Some(*index),
            Inner::NotConfirmed(_) => {
                if cfg!(debug_assertions) {
                    panic!("inner error: focused element not confirmed since last redrawing");
                }
                None
            }
            Inner::None => None,
        }
    }

    pub fn focus_on(&mut self, ed: EventDispatcher, index: usize) {
        self.0 = Inner::Confirmed {
            protected: Protected::new(ed),
            index,
        };
    }

    pub fn blur(&mut self) {
        self.0 = Inner::None;
    }

    pub fn to_not_confirmed(&mut self) {
        take_mut::take(&mut self.0, |mut this| {
            if let Inner::Confirmed { protected: ev, .. } = this {
                this = Inner::NotConfirmed(ev);
            }
            this
        });
    }

    pub fn drop_not_confirmed(&mut self) {
        if let Inner::NotConfirmed(_) = &self.0 {
            self.0 = Inner::None;
        }
    }

    pub fn try_confirm(&mut self, other: &EventDispatcher, index: usize) {
        take_mut::take(&mut self.0, |this| match this {
            Inner::NotConfirmed(protected) if protected.get().is_same(other) => {
                Inner::Confirmed { protected, index }
            }
            _ => this,
        });
    }
}
