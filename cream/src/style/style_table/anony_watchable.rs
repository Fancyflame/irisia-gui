use std::any::Any;

use crate::data_driven::{DataSource, Watchable};

pub(super) trait AnonymousWatchable {
    fn anonymous_get(&self) -> DataSource<dyn Any>;
}

impl<T> AnonymousWatchable for T
where
    T: Watchable,
    <T as Watchable>::Data: 'static,
{
    fn anonymous_get(&self) -> DataSource<dyn Any> {
        DataSource::map(self.get(), |x| x as &dyn Any)
    }
}
