use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    prim_element::{Element, WeakElement, layout::SpaceConstraint},
    primitive::Size,
};

pub(super) struct ReflowScheduler {
    pool: HashMap<*const (), ReflowNode>,
    key_list: Vec<*const ()>,
    // first_certain: Option<*const ()>,
}

struct ReflowNode {
    el: WeakElement,
}

impl ReflowScheduler {
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
            key_list: Vec::new(),
        }
    }

    fn load_keys(&mut self) {
        self.key_list.clear();
        self.key_list.extend(self.pool.keys());
    }

    pub fn insert(&mut self, el: &WeakElement) {
        self.pool
            .insert(el.as_ptr() as _, ReflowNode { el: el.clone() });
    }

    pub fn get_reflow_roots(&mut self) -> impl Iterator<Item = Element> {
        self.find_fixed_size_ancestors();
        self.remove_duplicated();
        dbg!(self.pool.len());

        self.pool
            .drain()
            .map(|(_, node)| node.el.upgrade().unwrap())
    }

    fn find_fixed_size_ancestors(&mut self) {
        self.load_keys();

        for key in self.key_list.iter() {
            let origin_node = match self.pool[key].el.upgrade() {
                Some(rc) => rc,
                None => {
                    self.pool.remove(key);
                    continue;
                }
            };

            let origin_node_key = get_key(&origin_node);
            let mut current_child_node = origin_node;

            let mut current_is_self = true;
            loop {
                if is_fixed_size(
                    &current_child_node
                        .borrow()
                        .common()
                        .layout_input
                        .unwrap()
                        .constraint,
                ) {
                    if !current_is_self {
                        self.pool.remove(&origin_node_key);
                        self.pool.insert(
                            get_key(&current_child_node),
                            ReflowNode {
                                el: Rc::downgrade(&current_child_node),
                            },
                        );
                    }
                    break;
                }

                current_is_self = false;

                if let Some(parent) = get_parent(&current_child_node) {
                    current_child_node = parent;
                }
            }
        }
    }

    fn remove_duplicated(&mut self) {
        self.load_keys();

        'next_node: for key in self.key_list.iter() {
            let mut current_child_node = match self.pool[key].el.upgrade() {
                Some(rc) => rc,
                None => {
                    self.pool.remove(key);
                    continue;
                }
            };

            while let Some(parent_node) = get_parent(&current_child_node) {
                if self.pool.contains_key(&get_key(&parent_node)) {
                    self.pool.remove(key);
                    continue 'next_node;
                }
                current_child_node = parent_node;
            }
        }
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }
}

fn is_fixed_size(constraint: &Size<SpaceConstraint>) -> bool {
    matches!(
        constraint,
        Size {
            width: SpaceConstraint::Exact(_),
            height: SpaceConstraint::Exact(_)
        }
    )
}

fn get_key(node: &Element) -> *const () {
    Rc::as_ptr(node) as _
}

pub fn get_parent(node: &Element) -> Option<Element> {
    node.borrow()
        .common()
        .ctx
        .parent
        .as_ref()
        .map(|weak| weak.upgrade().unwrap() as _)
}
