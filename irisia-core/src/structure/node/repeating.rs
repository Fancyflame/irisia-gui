use std::{collections::HashMap, hash::Hash};

use irisia_utils::ReuseVec;

use crate::{
    application::event_comp::NewPointerEvent,
    element::SelfCache,
    structure::{
        activate::{ActivatedStructure, CacheUpdateArguments, Structure, UpdateCache, Visit},
        cache::NodeCache,
        layer::LayerRebuilder,
    },
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

    fn activate(self, cache: &mut SelfCache<Self>) -> Self::Activated {
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
    K: Hash + Eq + 'static,
    T: ActivatedStructure,
{
    type Cache = RepeatingCache<K, T::Cache>;

    fn element_count(&self) -> usize {
        self.vectored.len()
    }
}

impl<K, T, V> Visit<V> for RepeatingActivated<K, T>
where
    K: Hash + Eq + 'static,
    T: Visit<V>,
{
    fn visit_at(&self, mut index: usize, visitor: &mut V) {
        for (_, x) in &self.vectored {
            x.visit_at(index, visitor);
            index += x.element_count();
        }
    }
}

impl<K, T, L> UpdateCache<L> for RepeatingActivated<K, T>
where
    K: Clone + Hash + Eq + 'static,
    T: UpdateCache<L>,
{
    fn update(mut self, args: CacheUpdateArguments<Self::Cache, L>) -> Result<bool> {
        let CacheUpdateArguments {
            mut offset,
            cache,
            global_content,
            layouter,
            equality_matters: mut unchange,
        } = args;

        cache.keys.clear();

        for (k, node) in self.vectored.drain(..) {
            let node_cache = match cache.element_cache.get_mut(&k) {
                Some(c) => c,
                None => inner_error!("element expected to be exists"),
            };

            node_cache.alive_signal = true;
            let element_count = node.element_count();
            unchange &= node.update(CacheUpdateArguments {
                offset,
                cache: &mut node_cache.value,
                global_content: global_content.downgrade_lifetime(),
                layouter,
                equality_matters: unchange,
            })?;

            offset += element_count;
            cache.keys.push(k);
        }

        cache.element_cache.retain(|_, cache_unit| {
            let sig = cache_unit.alive_signal;
            cache_unit.alive_signal = false;
            sig
        });

        cache.buffer = Some(self.vectored.into());

        Ok(unchange)
    }
}

// repeating cache

struct CacheUnit<T> {
    value: T,
    alive_signal: bool,
}

pub struct RepeatingCache<K, T> {
    element_cache: HashMap<K, CacheUnit<T>>,
    keys: Vec<K>,
    buffer: Option<ReuseVec>,
}

impl<K, T> Default for RepeatingCache<K, T> {
    fn default() -> Self {
        RepeatingCache {
            element_cache: HashMap::new(),
            keys: Vec::new(),
            buffer: None,
        }
    }
}

impl<K, T> NodeCache for RepeatingCache<K, T>
where
    K: Hash + Eq + 'static,
    T: NodeCache,
{
    fn render(&self, rebuilder: &mut LayerRebuilder) -> Result<()> {
        for key in self.keys.iter() {
            self.element_cache[key].value.render(rebuilder)?;
        }
        Ok(())
    }

    fn emit_event(&mut self, new_event: &NewPointerEvent) -> bool {
        let mut result = false;
        for key in self.keys.iter_mut().rev() {
            result |= self
                .element_cache
                .get_mut(&key)
                .unwrap()
                .value
                .emit_event(new_event);
        }
        result
    }
}
