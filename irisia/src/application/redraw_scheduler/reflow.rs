use std::{
    collections::{HashMap, hash_map::Entry},
    rc::{Rc, Weak},
};

use crate::{
    WeakHandle,
    prim_element::{Element, RenderTree},
};

pub(super) struct ReflowScheduler {
    pool: HashMap<*const (), ReflowNode>,
    // first_certain: Option<*const ()>,
}

struct ReflowNode {
    el: WeakHandle<dyn RenderTree>,
    sibling: Option<*const ()>,
    node_type: ReflowNodeType,
}

#[derive(Clone, Copy)]
enum ReflowNodeType {
    Certain,
    Virtual { children_header: *const () },
}

impl ReflowScheduler {
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    pub fn push_reflow(&mut self, leaf_element: &Element) {
        let mut current_searching_node = leaf_element.clone();

        // find if there is a certain ancestor exists
        loop {
            let key = get_key(&current_searching_node);

            match self.pool.get(&key) {
                Some(node) => match node.node_type {
                    ReflowNodeType::Certain { .. } => return,
                    ReflowNodeType::Virtual { .. } => break,
                },
                None => {}
            }

            if let Some(parent) = get_parent(&current_searching_node) {
                current_searching_node = parent;
            } else {
                break;
            }
        }

        // if we reached here, it says that none of the ancestors' type is certain

        let mut current_child_element = match self.pool.entry(get_key(leaf_element)) {
            Entry::Vacant(vac) => {
                vac.insert(ReflowNode {
                    el: Rc::downgrade(&leaf_element),
                    sibling: None,
                    node_type: ReflowNodeType::Certain,
                });
                leaf_element.clone()
            }
            Entry::Occupied(mut occ) => {
                let node = occ.get_mut();

                let ReflowNodeType::Virtual { children_header } =
                    std::mem::replace(&mut node.node_type, ReflowNodeType::Certain)
                else {
                    unreachable!();
                };

                self.remove_tree(children_header);
                return;
            }
        };

        while let Some(parent) = get_parent(&current_child_element) {
            let child_key = get_key(&current_child_element);
            match self.pool.entry(get_key(&parent)) {
                Entry::Occupied(mut occ) => {
                    let child_sibling_key = match &mut occ.get_mut().node_type {
                        ReflowNodeType::Virtual { children_header } => {
                            std::mem::replace(children_header, child_key)
                        }
                        ReflowNodeType::Certain => unreachable!(),
                    };

                    self.pool.get_mut(&child_key).unwrap().sibling = Some(child_sibling_key);
                    return;
                }
                Entry::Vacant(vac) => {
                    vac.insert(ReflowNode {
                        el: Rc::downgrade(&parent),
                        sibling: None,
                        node_type: ReflowNodeType::Virtual {
                            children_header: child_key,
                        },
                    });
                    current_child_element = parent;
                }
            }
        }
    }

    pub fn get_reflow_roots(&mut self) -> impl Iterator<Item = Element> {
        self.pool
            .drain()
            .filter_map(|(_, node)| match node.node_type {
                ReflowNodeType::Certain => node.el.upgrade(),
                ReflowNodeType::Virtual { .. } => None,
            })
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    fn remove_tree(&mut self, mut children_header: *const ()) {
        loop {
            let ReflowNode {
                el: _,
                sibling,
                node_type,
            } = self
                .pool
                .remove(&children_header)
                .expect("child node not exists");

            match node_type {
                ReflowNodeType::Certain => {}
                ReflowNodeType::Virtual { children_header } => {
                    self.remove_tree(children_header);
                }
            }

            if let Some(sibling) = sibling {
                children_header = sibling;
            } else {
                break;
            }
        }
    }
}

fn get_key(node: &Element) -> *const () {
    Rc::as_ptr(node) as _
}

pub fn get_parent(node: &Element) -> Option<Element> {
    node.borrow_mut()
        .common_mut()
        .ctx
        .parent
        .as_ref()
        .and_then(Weak::upgrade)
        .map(|rc| rc as _)
}
