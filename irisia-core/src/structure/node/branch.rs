use crate::{
    element::render_content::BareContent,
    structure::activate::{
        ActivateUpdateArguments, ActivatedStructure, Renderable, Structure, Visit,
    },
    Result,
};

#[derive(Default)]
pub struct BranchCache<T, U> {
    arm1: T,
    arm2: U,
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

    fn activate(
        self,
        cache: &mut <Self::Activated as ActivatedStructure>::Cache,
        content: &BareContent,
    ) -> Self::Activated {
        match self {
            Branch::Arm1(a) => Branch::Arm1(a.activate(&mut cache.arm1, content)),
            Branch::Arm2(a) => Branch::Arm2(a.activate(&mut cache.arm2, content)),
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

impl<T, U, L> Renderable<L> for Branch<T, U>
where
    T: Renderable<L>,
    U: Renderable<L>,
{
    fn update(self, args: ActivateUpdateArguments<Self::Cache, L>) -> Result<bool> {
        let ActivateUpdateArguments {
            offset,
            cache,
            bare_content,
            layouter,
            equality_matters,
        } = args;
        match self {
            Branch::Arm1(a) => a.update(ActivateUpdateArguments {
                offset,
                cache: &mut cache.arm1,
                bare_content,
                layouter,
                equality_matters,
            }),
            Branch::Arm2(a) => a.update(ActivateUpdateArguments {
                offset,
                cache: &mut cache.arm2,
                bare_content,
                layouter,
                equality_matters,
            }),
        }
    }
}
