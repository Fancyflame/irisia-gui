pub use updater::StateUpdate;

pub mod updater;

pub trait PropMaybeChanged<T> {
    fn take_change(self) -> Option<T>;
}

impl<T> PropMaybeChanged<T> for (T,) {
    fn take_change(self) -> Option<T> {
        Some(self.0)
    }
}

impl<T> PropMaybeChanged<T> for () {
    fn take_change(self) -> Option<T> {
        None
    }
}
