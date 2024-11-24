use std::rc::Weak;

use crate::hook::utils::WriteGuard;

use super::Inner;

pub struct EffectState<T>(Weak<dyn EffectInner<T>>);

impl<T> EffectState<T> {
    pub(super) fn new(weak: Weak<dyn EffectInner<T>>) -> Self {
        Self(weak)
    }

    pub fn valid(&self) -> bool {
        self.0.strong_count() != 0
    }

    pub fn update<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(WriteGuard<T>) -> R,
    {
        let rc = self.0.upgrade()?;
        let ret = f(rc.write_guard());
        Some(ret)
    }

    pub fn set(&self, value: T) -> bool {
        let Some(rc) = self.0.upgrade() else {
            return false;
        };
        *rc.write_guard() = value;
        true
    }
}

impl<T> Clone for EffectState<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub(super) trait EffectInner<T> {
    fn write_guard(&self) -> WriteGuard<T>;
}

impl<T, F, Fd: FnOnce(), D> EffectInner<T> for Inner<T, F, Fd, D> {
    fn write_guard(&self) -> WriteGuard<T> {
        WriteGuard::new(
            self.state
                .borrow_mut()
                .expect("you cannot update `Effect` when it is borrowed"),
            &self.listener_list,
        )
    }
}
