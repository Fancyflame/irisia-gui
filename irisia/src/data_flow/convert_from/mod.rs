mod auto_impl;

pub trait ConvertFrom<T> {
    fn create_from(value: T) -> Self;
    fn update_from(&mut self, value: T);
}

pub struct AssignTo<T>(T);

#[must_use]
pub struct Updater<'a, T>(pub(super) UpdaterInner<'a, T>);

pub(super) enum UpdaterInner<'a, T> {
    Unassigned(&'a mut Option<T>),
    OutOfDate { target: &'a mut T, updated: bool },
}

impl<'a, T> Updater<'a, T> {
    pub fn update<U>(mut self, data: U)
    where
        T: ConvertFrom<U>,
    {
        match &mut self.0 {
            UpdaterInner::Unassigned(opt) => **opt = Some(T::create_from(data)),
            UpdaterInner::OutOfDate { target, updated } => {
                target.update_from(data);
                *updated = true;
            }
        }
    }
}

impl<T> Drop for Updater<'_, T> {
    fn drop(&mut self) {
        let is_updated = match &self.0 {
            UpdaterInner::Unassigned(opt) => opt.is_some(),
            UpdaterInner::OutOfDate { updated, .. } => *updated,
        };

        if !is_updated {
            panic!("the updater must be used");
        }
    }
}
