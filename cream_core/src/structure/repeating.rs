use std::{
    collections::HashMap,
    hash::Hash,
    iter::{Flatten, Map},
    slice::Iter,
};

use smallvec::SmallVec;

use crate::{style::reader::StyleReader, Result};

use super::Node;

struct CacheUnit<T> {
    value: T,
    alive_signal: bool,
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
    K: Clone + Hash + Eq + Send + Sync + 'static,
    T: Node,
{
    type Cache = RepeatingCache<K, <T as Node>::Cache>;
    type Iter<'a, S> =
        Flatten<Map<Iter<'a, (K, T)>, fn(&'a (K, T)) -> <T as Node>::Iter<'a, S>>>
        where
            Self: 'a;

    fn style_iter<'a, S>(
        &'a self,
    ) -> Flatten<Map<Iter<(K, T)>, fn(&'a (K, T)) -> <T as Node>::Iter<'a, S>>>
    where
        S: StyleReader,
    {
        let func: fn(&'a (_, T)) -> _ = |(_, x)| x.style_iter::<S>();
        self.nodes.iter().map(func).flatten()
    }

    fn __finish_iter<S, F>(
        self,
        cache: &mut Self::Cache,
        mut content: crate::element::render_content::WildRenderContent,
        map: &mut F,
    ) -> crate::Result<()>
    where
        F: FnMut(S, Option<crate::primary::Region>) -> Result<crate::primary::Region>,
        S: StyleReader,
    {
        for (k, x) in self.nodes {
            match cache.0.get_mut(&k) {
                Some(c) => {
                    c.alive_signal = true;
                    x.__finish_iter(&mut c.value, content.downgrade_lifetime(), map)?
                }
                None => {
                    let mut value = T::Cache::default();
                    x.__finish_iter(&mut value, content.downgrade_lifetime(), map)?;
                    cache.0.insert(
                        k,
                        CacheUnit {
                            value,
                            alive_signal: true,
                        },
                    );
                }
            }
        }

        cache.0.retain(|_, cache_unit| {
            let sig = cache_unit.alive_signal;
            cache_unit.alive_signal = false;
            sig
        });

        Ok(())
    }
}
