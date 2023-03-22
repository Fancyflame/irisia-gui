use std::any::Any;

pub struct CacheBox(Option<Box<dyn Any + Send + Sync>>);

impl CacheBox {
    pub const fn new() -> Self {
        CacheBox(None)
    }

    pub(crate) fn get_cache<T>(&mut self) -> &mut T
    where
        T: Default + Send + Sync + 'static,
    {
        match &mut self.0 {
            inner @ None => {
                *inner = Some(Box::new(T::default()));
                inner.as_mut().unwrap().downcast_mut().unwrap()
            }
            Some(cache) => {
                if cache.is::<T>() {
                    cache.downcast_mut().unwrap()
                } else {
                    if cfg!(debug_assertions) {
                        panic!("one cache box can only store one cache");
                    } else {
                        *cache = Box::new(T::default());
                        cache.downcast_mut().unwrap()
                    }
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
