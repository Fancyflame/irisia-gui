use super::{StateStore, StateUpdate};

/// Updater Difference Watcher
pub struct Udw<T: StateStore> {
    update_place: T::Store,
    value: T::Store,
    changed: bool,
}

impl<T> Default for Udw<T>
where
    T: StateStore,
{
    fn default() -> Self {
        Self {
            update_place: <_>::default(),
            value: <_>::default(),
            changed: false,
        }
    }
}

impl<T: StateStore> StateStore for Udw<T> {
    type Store = Self;
}

impl<T, U> StateUpdate<U> for Udw<T>
where
    T: StateUpdate<U>,
    T::Store: PartialEq<T::Store>,
{
    fn state_update(state: &mut Self::Store, updater: U) {
        T::state_update(&mut state.update_place, updater);
        state.changed = state.update_place == state.value;
        std::mem::swap(&mut state.update_place, &mut state.value);
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

    pub fn set_unchanged(&mut self) {
        self.changed = false;
    }
}

/// General Difference Watcher
#[derive(Default)]
pub struct Gdw<T> {
    value: T,
    changed: bool,
}

impl<T: Default + 'static> StateStore for Gdw<T> {
    type Store = Self;
}

impl<T> StateUpdate<T> for Gdw<T>
where
    T: Default + 'static + PartialEq<T>,
{
    fn state_update(state: &mut Self::Store, updater: T) {
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

    pub fn set_unchanged(&mut self) {
        self.changed = false;
    }
}
