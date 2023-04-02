use std::any::Any;

pub struct CacheBox(Option<Box<dyn Any + 'static>>);

impl CacheBox {
    pub const fn new() -> Self {
        CacheBox(None)
    }

    /// Stores the cache of a node cahce of **one type**. If
    /// different types detected, it will panic under debug mode
    /// and initialize a new cache under release mode, which will
    /// heavily effect the performance.
    pub(crate) fn get_cache<T>(&mut self) -> &mut T
    where
        T: Default + 'static,
    {
        match &mut self.0 {
            inner @ None => {
                *inner = Some(Box::<T>::default());
                inner.as_mut().unwrap().downcast_mut().unwrap()
            }
            Some(cache) => {
                if cache.is::<T>() {
                    cache.downcast_mut().unwrap()
                } else if cfg!(debug_assertions) {
                    panic!("one cache box can only store one type of cache");
                } else {
                    *cache = Box::<T>::default();
                    cache.downcast_mut().unwrap()
                }
            }
        }
    }
}

impl Default for CacheBox {
    fn default() -> Self {
        Self::new()
    }
}
