use std::{collections::HashMap, hash::Hash};

use irisia_utils::ReuseVec;

use crate::{
    structure::visit::{ActivatedStructure, BareContentWrapper, Renderable, Structure, Visit},
    Result,
};

// repeating structure

pub struct Repeating<I> {
    node_iter: I,
}

impl<I: Iterator> Repeating<I> {
    pub fn new(iter: I) -> Self {
        Repeating { node_iter: iter }
    }
}

impl<K, T, I> Structure for Repeating<I>
where
    I: Iterator<Item = (K, T)>,
    K: Clone + Hash + Eq + 'static,
    T: Structure,
{
    type Activated = RepeatingActivated<K, T::Activated>;

    fn activate(
        self,
        cache: &mut <Self::Activated as ActivatedStructure>::Cache,
        content: &BareContentWrapper,
    ) -> Self::Activated {
        let mut buffer = match cache.buffer.take() {
            Some(rv) => {
                if cfg!(debug_assertions) {
                    rv.try_into_vec()
                        .unwrap_or_else(|_| inner_error!("cannot reuse the vector"))
                } else {
                    rv.into_vec()
                }
            }
            None => Vec::new(),
        };

        buffer.extend(self.node_iter.map(|(k, n)| {
            (
                k.clone(),
                n.activate(
                    &mut cache
                        .element_cache
                        .entry(k)
                        .or_insert_with(|| CacheUnit {
                            value: <_>::default(),
                            alive_signal: true,
                        })
                        .value,
                    content,
                ),
            )
        }));

        RepeatingActivated { vectored: buffer }
    }
}

// repeating activated

pub struct RepeatingActivated<K, T> {
    vectored: Vec<(K, T)>,
}

impl<K, T> ActivatedStructure for RepeatingActivated<K, T>
where
    K: 'static,
    T: ActivatedStructure,
{
    type Cache = RepeatingCache<K, T::Cache>;

    fn element_count(&self) -> usize {
        self.vectored.len()
    }
}

impl<K, T, V> Visit<V> for RepeatingActivated<K, T>
where
    K: 'static,
    T: Visit<V>,
{
    fn visit_at(&self, mut index: usize, visitor: &mut V) {
        for (_, x) in &self.vectored {
            x.visit_at(index, visitor);
            index += x.element_count();
        }
    }
}

impl<K, T, L> Renderable<L> for RepeatingActivated<K, T>
where
    K: Clone + Hash + Eq + 'static,
    T: Renderable<L>,
{
    fn render_at(
        mut self,
        mut index: usize,
        cache: &mut Self::Cache,
        mut bare_content: BareContentWrapper,
        layouter: &mut L,
    ) -> Result<()> {
        for (k, node) in self.vectored.drain(..) {
            let node_cache = match cache.element_cache.get_mut(&k) {
                Some(c) => c,
                None => inner_error!("element expected to be exists"),
            };

            node_cache.alive_signal = true;
            let element_count = node.element_count();
            node.render_at(
                index,
                &mut node_cache.value,
                BareContentWrapper(bare_content.0.downgrade_lifetime()),
                layouter,
            )?;

            index += element_count;
        }

        cache.element_cache.retain(|_, cache_unit| {
            let sig = cache_unit.alive_signal;
            cache_unit.alive_signal = false;
            sig
        });

        cache.buffer = Some(self.vectored.into());

        Ok(())
    }
}

// repeating cache

struct CacheUnit<T> {
    value: T,
    alive_signal: bool,
}

pub struct RepeatingCache<K, T> {
    element_cache: HashMap<K, CacheUnit<T>>,
    buffer: Option<ReuseVec>,
}

impl<K, T> Default for RepeatingCache<K, T> {
    fn default() -> Self {
        RepeatingCache {
            element_cache: HashMap::new(),
            buffer: None,
        }
    }
}
