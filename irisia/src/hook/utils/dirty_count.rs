use std::cell::Cell;

use crate::hook::listener::CallbackAction;

/// Use it when implementing a consumer
pub struct DirtyCount {
    dirty_stack: Cell<u32>,
    need_update: Cell<bool>,
    allow_push: Cell<bool>,
}

impl DirtyCount {
    pub fn new() -> Self {
        Self {
            dirty_stack: Cell::new(0),
            need_update: Cell::new(false),
            allow_push: Cell::new(true),
        }
    }

    /// return the action you may need to spread to all of your dependents
    pub fn push(&self, action: CallbackAction) -> Option<CallbackAction> {
        match action {
            CallbackAction::Update => self.need_update.set(true),
            CallbackAction::ClearDirty => {}
            CallbackAction::RegisterDirty => {
                if !self.allow_push.get() {
                    panic!(
                        "registering dirty is not allowed until all dependencies \
                        are clean (updated or cleared dirty)"
                    );
                }
                let count = self.dirty_stack.get().checked_add(1).unwrap();
                self.dirty_stack.set(count);

                return (count == 1).then_some(CallbackAction::RegisterDirty);
            }
        }

        self.allow_push.set(false);
        let count = self
            .dirty_stack
            .get()
            .checked_sub(1)
            .expect("cannot decrease dirty count as which is 0");
        self.dirty_stack.set(count);

        if count != 0 {
            return None;
        }

        self.allow_push.set(true);
        if self.need_update.replace(false) {
            Some(CallbackAction::Update)
        } else {
            Some(CallbackAction::ClearDirty)
        }
    }
}
