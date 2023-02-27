use std::{
    collections::HashMap,
    hash::Hash,
    iter::{Flatten, Map},
    slice::Iter,
};

use smallvec::SmallVec;

use crate::{element::RenderContent, style::reader::StyleReader};

use super::Node;

struct CacheUnit<T> {
    value: T,
    used_signal: bool,
}

pub struct RepeatingCache<K, T>(HashMap<K, CacheUnit<T>>);

impl<K, T> Default for RepeatingCache<K, T> {
    fn default() -> Self {
        RepeatingCache(HashMap::new())
    }
}

pub struct Repeating<K, T> {
    nodes: SmallVec<[(K, T); 20]>,
}

impl<K, T> Repeating<K, T> {
    pub fn new<I>(iter: I) -> Self
    where
        I: Iterator<Item = (K, T)>,
    {
        Repeating {
            nodes: iter.collect(),
        }
    }
}

impl<K, T> Node for Repeating<K, T>
where
    K: Clone + Hash + Eq + 'static,
    T: Node,
{
    type Cache = RepeatingCache<K, <T as Node>::Cache>;
    type StyleIter<'a, S> =
        Flatten<Map<Iter<'a, (K, T)>, fn(&'a (K, T)) -> <T as Node>::StyleIter<'a, S>>>
        where
            Self: 'a;

    fn style_iter<'a, S>(
        &'a self,
    ) -> Flatten<Map<Iter<(K, T)>, fn(&'a (K, T)) -> <T as Node>::StyleIter<'a, S>>>
    where
        S: StyleReader,
    {
        let func: fn(&'a (_, T)) -> _ = |(_, x)| x.style_iter::<S>();
        self.nodes.iter().map(func).flatten()
    }

    fn finish<'a, I>(self, cache: &mut Self::Cache, mut iter: I) -> crate::Result<()>
    where
        I: Iterator<Item = RenderContent<'a>>,
    {
        for (k, x) in self.nodes {
            match cache.0.get_mut(&k) {
                Some(c) => {
                    c.used_signal = true;
                    x.finish(&mut c.value, &mut iter)?
                }
                None => {
                    let mut value = T::Cache::default();
                    x.finish(&mut value, &mut iter)?;
                    cache.0.insert(
                        k,
                        CacheUnit {
                            value,
                            used_signal: true,
                        },
                    );
                }
            }
        }

        cache.0.retain(|_, cache_unit| {
            let sig = cache_unit.used_signal;
            cache_unit.used_signal = false;
            sig
        });

        Ok(())
    }
}
