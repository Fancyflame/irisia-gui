use std::{any::TypeId, collections::hash_map::Entry, rc::Rc};

use anyhow::{Result, anyhow, bail};
use smallvec::{SmallVec, smallvec};

use crate::{
    Component,
    global::{
        CompStock, EntityData, EntityObject, Runtime,
        components::{CompId, CompRef, CompRefMut},
        pool::PoolId,
    },
};

impl EntityObject {
    pub fn attach_component<T: Component>(&self, comp: T) -> Result<()> {
        let rt = self
            .rt
            .upgrade()
            .ok_or_else(|| anyhow!("runtime of this entity has destroyed"))?;

        let mut entity = rt.entities.borrow().access_mut(self.id).unwrap();

        let Entry::Vacant(vac) = entity.components.entry(TypeId::of::<T>()) else {
            bail!("cannot attach duplicate components of the same type");
        };

        let comp_pid = rt.components.borrow_mut().insert(comp).pid;
        vac.insert(comp_pid);
        Ok(())
    }

    fn get_comp_id<T: Component>(&self) -> Option<(Rc<Runtime>, CompId)> {
        let rt = self.rt.upgrade()?;

        let entity = rt.entities.borrow().access(self.id).ok()?;

        let tid = TypeId::of::<T>();
        let pid = entity.components.get(&tid)?.clone();

        Some((rt, CompId { pid, tid }))
    }

    pub fn get_component<T: Component>(&self) -> Option<CompRef<T>> {
        let (rt, comp_id) = self.get_comp_id::<T>()?;
        rt.components
            .borrow()
            .get(&comp_id)
            .expect("cannot borrow component as immutable because it has been borrowed as mutable")
    }

    pub fn get_component_mut<T: Component>(&self) -> Option<CompRefMut<T>> {
        let (rt, comp_id) = self.get_comp_id::<T>()?;
        rt.components
            .borrow()
            .get_mut(&comp_id)
            .expect("cannot borrow component as mutable because it has been borrowed")
    }
}

impl Runtime {
    /// 删除当前实体及其所有子节点，不修复父级和兄弟级关系
    pub(crate) fn destroy_entity(&self, entity_id: PoolId) {
        let mut entities = self.entities.borrow_mut();
        let Some(root) = entities.remove(entity_id, false) else {
            return;
        };
        let mut queue: SmallVec<[EntityData; 16]> = smallvec![root];

        while let Some(data) = queue.pop() {
            for (tid, pid) in data.components {
                let comp_id = CompId { tid, pid };
                CompStock::remove(self.components.borrow_mut(), &comp_id);
            }

            let mut entity_to_destroy = data.first_child;
            while let Some(child) = entity_to_destroy {
                let child_data = entities
                    .remove(child.id, false)
                    .expect("child entity should exist while destroying subtree");
                entity_to_destroy = child_data.next_sibling.as_ref().cloned();
                queue.push(child_data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, rc::Rc};

    use super::*;

    struct TestComponent(u32);

    impl Component for TestComponent {}

    fn entity_ref(rt: &Rc<Runtime>, id: PoolId) -> EntityObject {
        EntityObject {
            rt: Rc::downgrade(rt),
            id,
        }
    }

    fn insert_entity(rt: &Rc<Runtime>) -> EntityObject {
        let id = rt.entities.borrow_mut().insert(EntityData {
            components: HashMap::new(),
            first_child: None,
            next_sibling: None,
        });

        entity_ref(rt, id)
    }

    fn link_children(rt: &Rc<Runtime>, parent: &EntityObject, children: &[&EntityObject]) {
        let entities = rt.entities.borrow_mut();
        entities.access_mut(parent.id).unwrap().first_child =
            children.first().map(|child| entity_ref(rt, child.id));

        for (index, child) in children.iter().enumerate() {
            let next = children
                .get(index + 1)
                .map(|next_child| entity_ref(rt, next_child.id));

            let mut data = entities.access_mut(child.id).unwrap();
            data.next_sibling = next;
        }
    }

    #[test]
    fn destroy_entity_removes_root_and_entire_subtree() {
        let rt = Runtime::new();
        let root = insert_entity(&rt);
        let child_1 = insert_entity(&rt);
        let child_2 = insert_entity(&rt);
        let grandchild_1 = insert_entity(&rt);
        let grandchild_2 = insert_entity(&rt);

        link_children(&rt, &root, &[&child_1, &child_2]);
        link_children(&rt, &child_1, &[&grandchild_1, &grandchild_2]);

        rt.destroy_entity(root.id);

        let entities = rt.entities.borrow();
        assert!(entities.access(root.id).is_err());
        assert!(entities.access(child_1.id).is_err());
        assert!(entities.access(child_2.id).is_err());
        assert!(entities.access(grandchild_1.id).is_err());
        assert!(entities.access(grandchild_2.id).is_err());
    }

    #[test]
    fn destroy_entity_delays_component_drop_for_removed_entities() {
        let rt = Runtime::new();
        let root = insert_entity(&rt);
        let child = insert_entity(&rt);
        link_children(&rt, &root, &[&child]);

        root.attach_component(TestComponent(3)).unwrap();
        child.attach_component(TestComponent(7)).unwrap();

        let root_comp_id = root.get_comp_id::<TestComponent>().unwrap().1;
        let comp_id = child.get_comp_id::<TestComponent>().unwrap().1;
        let held_root_ref = root.get_component::<TestComponent>().unwrap();
        let held_ref = child.get_component::<TestComponent>().unwrap();

        rt.destroy_entity(root.id);

        let kept_root_alive = rt
            .components
            .borrow()
            .get::<TestComponent>(&root_comp_id)
            .unwrap()
            .unwrap();
        let kept_alive = rt
            .components
            .borrow()
            .get::<TestComponent>(&comp_id)
            .unwrap()
            .unwrap();
        assert_eq!(kept_root_alive.0, 3);
        assert_eq!(kept_alive.0, 7);

        drop(kept_root_alive);
        drop(kept_alive);
        drop(held_root_ref);
        drop(held_ref);

        assert!(
            rt.components
                .borrow()
                .get::<TestComponent>(&root_comp_id)
                .unwrap()
                .is_none()
        );
        assert!(
            rt.components
                .borrow()
                .get::<TestComponent>(&comp_id)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn destroy_entity_handles_deep_subtree_iteratively() {
        let rt = Runtime::new();
        let root = insert_entity(&rt);
        let mut removed_ids = vec![];
        let mut parent = root.id;

        for _ in 0..10_000 {
            let child = insert_entity(&rt);
            {
                let entities = rt.entities.borrow_mut();
                entities.access_mut(parent).unwrap().first_child = Some(entity_ref(&rt, child.id));

                let mut child_data = entities.access_mut(child.id).unwrap();
                child_data.next_sibling = None;
            }

            parent = child.id;
            removed_ids.push(child.id);
        }

        rt.destroy_entity(root.id);

        let entities = rt.entities.borrow();
        assert!(entities.access(root.id).is_err());
        for id in removed_ids {
            assert!(entities.access(id).is_err());
        }
    }

    #[test]
    fn destroy_entity_is_noop_for_leaf_and_already_removed_entities() {
        let rt = Runtime::new();
        let entity = insert_entity(&rt);

        rt.destroy_entity(entity.id);
        rt.destroy_entity(entity.id);

        let entities = rt.entities.borrow();
        assert!(entities.access(entity.id).is_err());
    }
}
