use super::{StateStore, StateUpdate};

/// Updater Difference Watcher
pub struct Udw<T: StateStore> {
    old: Option<T::Store>,
    value: T::Store,
    changed: bool,
}

impl<T: StateStore> StateStore for Udw<T> {
    type Store = Self;
}

impl<T, U> StateUpdate<U> for Udw<T>
where
    T: StateUpdate<U>,
    T::Store: PartialEq<T::Store>,
{
    fn state_created(updater: U) -> Self {
        Udw {
            old: None,
            value: T::state_created(updater),
            changed: true,
        }
    }

    fn state_changed(state: &mut Self, updater: U) {
        let old_place = match &mut state.old {
            Some(recorder) => {
                T::state_changed(recorder, updater);
                recorder
            }
            place @ None => place.insert(T::state_created(updater)),
        };

        std::mem::swap(old_place, &mut state.value);
        state.changed = *old_place == state.value;
    }
}

impl<T: StateStore> Udw<T> {
    pub fn get(&self) -> &T::Store {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut T::Store {
        self.changed = true;
        &mut self.value
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }
}

/// General Difference Watcher
pub struct Gdw<T> {
    value: T,
    changed: bool,
}

impl<T: 'static> StateStore for Gdw<T> {
    type Store = Self;
}

impl<T> StateUpdate<T> for Gdw<T>
where
    T: PartialEq<T> + 'static,
{
    fn state_created(updater: T) -> Self {
        Gdw {
            value: updater,
            changed: true,
        }
    }

    fn state_changed(state: &mut Self, updater: T) {
        state.changed = updater == state.value;
        state.value = updater;
    }
}

impl<T> Gdw<T> {
    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.changed = true;
        &mut self.value
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }
}
