use anyhow::Result;
use smallvec::SmallVec;

use crate::structure::{element::UpdateAndCreate, Element};

pub struct UnkeyedStorage<T> {
    counter: usize,
    storage: SmallVec<[T; 1]>,
}

impl<E> UnkeyedStorage<E>
where
    E: Element,
{
    pub fn new() -> Self {
        UnkeyedStorage {
            counter: 0,
            storage: SmallVec::new(),
        }
    }

    pub fn start(&mut self) {
        self.counter = 0;
    }

    pub fn update<A>(&mut self, args: A) -> Result<&E>
    where
        E: UpdateAndCreate<A>,
    {
        let ele = if self.counter < self.storage.len() {
            let e = self.storage.get_mut(self.counter).unwrap();
            e.update(args)?;
            &*e
        } else {
            self.storage.push(E::create(args)?);
            self.storage.last().unwrap()
        };

        self.counter += 1;
        Ok(ele)
    }
}
