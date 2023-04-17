use std::{collections::HashMap, hash::Hash, iter::FlatMap, slice::Iter};

use smallvec::SmallVec;

use crate::{style::reader::StyleReader, Result};

use super::{node::BareContentWrapper, RenderingNode, VisitIter};

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

impl<K, T> RenderingNode for Repeating<K, T>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    T: RenderingNode,
{
    type Cache = RepeatingCache<K, T::Cache>;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: &BareContentWrapper) {
        for (k, x) in &mut self.nodes {
            match cache.0.get_mut(k) {
                Some(c) => {
                    c.alive_signal = true;
                    x.prepare_for_rendering(&mut c.value, content)
                }
                None => {
                    let mut value = T::Cache::default();
                    x.prepare_for_rendering(&mut value, content);
                    cache.0.insert(
                        k.clone(),
                        CacheUnit {
                            value,
                            alive_signal: true,
                        },
                    );
                }
            }
        }
    }

    fn element_count(&self) -> usize {
        self.nodes.len()
    }

    fn finish<S, F>(
        self,
        cache: &mut Self::Cache,
        mut content: BareContentWrapper,
        map: &mut F,
    ) -> crate::Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<crate::primary::Region>,
        S: StyleReader,
    {
        for (k, x) in self.nodes {
            x.finish(
                &mut cache.0.get_mut(&k).unwrap().value,
                BareContentWrapper(content.0.downgrade_lifetime()),
                map,
            )?;
        }

        cache.0.retain(|_, cache_unit| {
            let sig = cache_unit.alive_signal;
            cache_unit.alive_signal = false;
            sig
        });

        Ok(())
    }
}

impl<K, T, Prop> VisitIter<Prop> for Repeating<K, T>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    T: VisitIter<Prop>,
{
    type VisitIter<'a, S> =
        FlatMap<Iter<'a,(K, T)>, T::VisitIter<'a,S>, fn(&'a (K, T)) -> T::VisitIter<'a,S>>
        where
            S:StyleReader,
            Self: 'a;

    fn visit_iter<'a, S>(&'a self) -> Self::VisitIter<'a, S>
    where
        S: StyleReader,
    {
        let func: fn(&'a (_, T)) -> _ = |(_, x)| x.visit_iter::<S>();
        self.nodes.iter().flat_map(func)
    }
}
