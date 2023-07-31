use anyhow::anyhow;

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

#[derive(Default)]
pub struct BranchCache<T, U> {
    arm1: T,
    arm2: U,
    current: CurrentArm,
}

#[derive(Default, PartialEq, Eq)]
enum CurrentArm {
    #[default]
    NotInit,

    Arm1,
    Arm2,
}

pub enum Branch<T, U> {
    Arm1(T),
    Arm2(U),
}

impl<T, U> Structure for Branch<T, U>
where
    T: Structure,
    U: Structure,
{
    type Activated = Branch<T::Activated, U::Activated>;

    fn activate(self, cache: &mut SelfCache<Self>) -> Self::Activated {
        match self {
            Branch::Arm1(a) => Branch::Arm1(a.activate(&mut cache.arm1)),
            Branch::Arm2(a) => Branch::Arm2(a.activate(&mut cache.arm2)),
        }
    }
}

impl<T, U> ActivatedStructure for Branch<T, U>
where
    T: ActivatedStructure,
    U: ActivatedStructure,
{
    type Cache = BranchCache<T::Cache, U::Cache>;

    fn element_count(&self) -> usize {
        match self {
            Branch::Arm1(a) => a.element_count(),
            Branch::Arm2(a) => a.element_count(),
        }
    }
}

impl<T, U, V> Visit<V> for Branch<T, U>
where
    T: Visit<V>,
    U: Visit<V>,
{
    fn visit_at(&self, index: usize, visitor: &mut V) {
        match self {
            Branch::Arm1(a) => a.visit_at(index, visitor),
            Branch::Arm2(a) => a.visit_at(index, visitor),
        }
    }
}

impl<T, U, L> UpdateCache<L> for Branch<T, U>
where
    T: UpdateCache<L>,
    U: UpdateCache<L>,
{
    fn update(self, args: CacheUpdateArguments<Self::Cache, L>) -> Result<bool> {
        let CacheUpdateArguments {
            offset,
            cache,
            global_content,
            layouter,
            equality_matters: mut unchange,
        } = args;

        match self {
            Branch::Arm1(a) => {
                unchange &= cache.current == CurrentArm::Arm1;
                cache.current = CurrentArm::Arm1;

                unchange &= a.update(CacheUpdateArguments {
                    offset,
                    cache: &mut cache.arm1,
                    global_content,
                    layouter,
                    equality_matters: unchange,
                })?;
            }
            Branch::Arm2(a) => {
                unchange &= cache.current == CurrentArm::Arm2;
                cache.current = CurrentArm::Arm2;

                unchange &= a.update(CacheUpdateArguments {
                    offset,
                    cache: &mut cache.arm2,
                    global_content,
                    layouter,
                    equality_matters: unchange,
                })?;
            }
        }

        Ok(unchange)
    }
}

impl<T, U> NodeCache for BranchCache<T, U>
where
    T: NodeCache,
    U: NodeCache,
{
    fn render(&self, rebuilder: &mut LayerRebuilder) -> Result<()> {
        match self.current {
            CurrentArm::Arm1 => self.arm1.render(rebuilder),
            CurrentArm::Arm2 => self.arm2.render(rebuilder),
            CurrentArm::NotInit => Err(anyhow!("this branch is not initialized")),
        }
    }

    fn emit_event(&mut self, new_event: &NewPointerEvent) -> bool {
        match self.current {
            CurrentArm::Arm1 => self.arm1.emit_event(new_event),
            CurrentArm::Arm2 => self.arm2.emit_event(new_event),
            CurrentArm::NotInit => false,
        }
    }
}
