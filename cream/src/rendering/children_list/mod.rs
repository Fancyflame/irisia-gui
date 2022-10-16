use std::rc::Rc;

use crate::{
    event::{callback::IntoCallback, Event},
    structure::{
        element::{ElementHandle, RcHandle},
        Element,
    },
    style::ApplyStyle,
};

use self::iter_by_render_order::Iter as IterByRenderOrder;

//pub mod children_iter;
pub mod iter_by_render_order;

pub struct ChildrenList {
    list: Vec<ItemInner>,
}

pub(self) struct ItemInner {
    value: ElementHandle,
    expanded: bool,
    children_head: Option<usize>,
    children_tail: Option<usize>,
    parent: Option<usize>,
    next: Option<usize>,
}

impl ChildrenList {
    pub fn new() -> Self {
        ChildrenList { list: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    fn append_child(&mut self, append_to: usize, ele: ElementHandle) -> usize {
        let index = self.list.len();
        let item = ItemInner {
            value: ele,
            expanded: false,
            children_head: None,
            children_tail: None,
            parent: Some(append_to),
            next: None,
        };

        self.list.push(item);
        self.append_group(append_to, index, index);

        index
    }

    fn append_group(&mut self, append_to: usize, head: usize, tail: usize) {
        let ele = &mut self.list[append_to];

        match &mut ele.children_tail {
            None => {
                ele.children_head = Some(head);
                ele.children_tail = Some(tail);
            }
            Some(ele_tail) => {
                let old_tail = *ele_tail;
                *ele_tail = tail;
                self.list[old_tail].next = Some(head);
            }
        }
    }

    fn get_child(&self, index: usize) -> &ItemInner {
        &self.list[index]
    }

    fn get_child_mut(&mut self, index: usize) -> &mut ItemInner {
        &mut self.list[index]
    }

    pub(super) fn iter_by_render_order(&self) -> IterByRenderOrder {
        IterByRenderOrder::new(self)
    }

    pub fn expand_tree<E: Element>(&mut self, ele: RcHandle<E>) {
        self.clear();
        self.list.push(ItemInner {
            value: ele as _,
            expanded: false,
            children_head: None,
            children_tail: None,
            parent: None,
            next: None,
        });

        let mut index = 0;
        while let Some(item) = self.list.get_mut(index) {
            index += 1;
            if item.expanded {
                continue;
            }
            // This element needs expanding

            item.expanded = true;
            let slot = Some((item.children_head.take(), item.children_tail.take()));
            let item = item.value.clone();

            let exp_tree = ExpandTree {
                list: self,
                root: index,
                index,
                slot,
            };

            item.borrow().expand_tree(exp_tree);
        }
    }
}

pub struct ExpandTree<'a> {
    list: &'a mut ChildrenList,
    root: usize,
    index: usize,
    slot: Option<(Option<usize>, Option<usize>)>,
}

impl ExpandTree<'_> {
    pub fn push_child(&mut self, ele: ElementHandle) -> &mut Self {
        ele.borrow_mut().service_mut().reset();
        self.index = self.list.append_child(self.index, ele);
        self
    }

    pub fn pop_child(&mut self) -> &mut Self {
        if self.index == self.root {
            panic!("cannot back to the upper level because the root is reached");
        }

        let this = self.list.get_child(self.index);

        let mut borrow_mut = this.value.borrow_mut();
        let this_children = borrow_mut.service_mut().children_mut();
        let mut next_child = this.children_head;
        while let Some(i) = next_child {
            let item = self.list.get_child(i);
            next_child = item.next;
            this_children.push(item.value.clone());
        }

        self.index = this.parent.unwrap();
        drop(borrow_mut);
        self
    }

    pub fn append_slot(&mut self) -> &mut Self {
        let slot = self.slot.take().expect("slot can only be use once");
        if let (Some(head), Some(tail)) = slot {
            self.list.append_group(self.index, head, tail);
        }
        self
    }

    pub fn apply_style<S: ApplyStyle>(&mut self, styles: &S) -> &mut Self {
        styles.apply_style(
            self.list
                .get_child_mut(self.index)
                .value
                .borrow_mut()
                .service_mut()
                .style_mut(),
        );
        self
    }

    pub fn apply_listener<Ev, I>(&mut self, ev: Ev, listener: Rc<I>) -> &mut Self
    where
        Ev: Event,
        I: IntoCallback<Ev::Arg>,
    {
        self.list
            .get_child_mut(self.index)
            .value
            .borrow_mut()
            .service_mut()
            .event_target_mut()
            .on(ev, listener);
        self
    }
}
