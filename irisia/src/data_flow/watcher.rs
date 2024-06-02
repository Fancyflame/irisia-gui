use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use super::{Listener, Wakeable};

#[derive(Clone, Copy)]
enum State {
    MultiTimes,
    Once,
    Discard,
}

struct WatcherCore<F> {
    func: RefCell<F>,
    handle: Handle,
    state: Cell<State>,
}

impl<F> Wakeable for WatcherCore<F>
where
    F: FnMut(&Handle),
{
    fn update(self: Rc<Self>) -> bool {
        match self.state.get() {
            State::MultiTimes => {
                self.func.borrow_mut()(&self.handle);
                true
            }

            State::Once => {
                self.state.set(State::Discard);
                self.func.borrow_mut()(&self.handle);
                false
            }

            State::Discard => false,
        }
    }
}

pub fn watcher<F>(mut watch_fn: F, call_immediately: bool) -> (Listener, Handle)
where
    F: FnMut(&Handle) + 'static,
{
    let rc = Rc::new_cyclic(|weak: &Weak<WatcherCore<F>>| {
        let handle = Handle(weak.clone());

        if call_immediately {
            watch_fn(&handle);
        }

        WatcherCore {
            func: RefCell::new(watch_fn),
            handle,
            state: Cell::new(State::MultiTimes),
        }
    });

    let handle = rc.handle.clone();
    (Listener::Rc(rc), handle)
}

#[derive(Clone)]
pub struct Handle(Weak<dyn SetState>);

trait SetState {
    fn set_state(&self) -> &Cell<State>;
}

impl<F> SetState for WatcherCore<F> {
    fn set_state(&self) -> &Cell<State> {
        &self.state
    }
}

impl Handle {
    fn set<F>(&self, f: F)
    where
        F: FnOnce(&Cell<State>),
    {
        if let Some(this) = self.0.upgrade() {
            f(this.set_state());
        }
    }

    pub fn discard(&self) {
        self.set(|cell| cell.set(State::Discard));
    }

    pub fn once(&self) {
        self.set(|cell| {
            if let State::MultiTimes = cell.get() {
                cell.set(State::Once);
            }
        });
    }
}
