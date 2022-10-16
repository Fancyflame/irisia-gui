use crate::structure::element::ElementHandle;

use super::ItemInner;

pub struct ChildrenIter<'a> {
    list: &'a Vec<ItemInner>,
    next: Option<usize>,
}

impl<'a> ChildrenIter<'a> {
    pub(super) fn new(list: &'a Vec<ItemInner>, parent: &'a ItemInner) -> Self {
        ChildrenIter {
            list: &list,
            next: parent.children_head,
        }
    }
}

impl<'a> Iterator for ChildrenIter<'a> {
    type Item = &'a ElementHandle;

    fn next(&mut self) -> Option<Self::Item> {
        let this = &self.list[self.next?];
        self.next = this.next;
        Some(&this.value)
    }
}
