use std::iter::FusedIterator;

use crate::structure::element::ElementHandle;

use super::{ChildrenList, ItemInner};

#[derive(Clone)]
pub struct Iter<'a> {
    list: &'a Vec<ItemInner>,
    next: Option<usize>,
}

impl<'a> Iter<'a> {
    pub(super) fn new(list: &'a ChildrenList) -> Self {
        Iter {
            list: &list.list,
            next: if list.list.len() != 0 { Some(0) } else { None },
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a ElementHandle;
    fn next(&mut self) -> Option<Self::Item> {
        let current = &self.list[self.next?];

        // If this element have children, the next one is the first kid.
        // If not, look into its next sibling, if still havn't, look into
        // its parent's next sibling, and so on up to the current
        // one has no parent, then we reached the end of the children
        // list.
        self.next = match current.children_head {
            next @ Some(_) => next,
            None => {
                let mut this = current;
                loop {
                    match this.next {
                        next @ Some(_) => break next,
                        None => match this.parent {
                            Some(parent) => this = &self.list[parent],
                            None => break None,
                        },
                    }
                }
            }
        };

        Some(&current.value)
    }
}

impl FusedIterator for Iter<'_> {}
