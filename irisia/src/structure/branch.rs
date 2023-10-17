use crate::{update_with::SpecificUpdate, Result};

use super::{MapVisit, UpdateWith, Visit, VisitLen, VisitMut};

use self::BranchModel::*;

pub enum Branch<A, B> {
    ArmA(A),
    ArmB(B),
}

pub enum BranchModel<A, B> {
    ArmA { value: A, other: Option<B> },
    ArmB { value: B, other: Option<A> },
}

impl<A, B, V> MapVisit<V> for Branch<A, B>
where
    A: MapVisit<V>,
    B: MapVisit<V>,
{
    type Output = Branch<A::Output, B::Output>;
    fn map(self, visitor: &V) -> Self::Output {
        match self {
            Branch::ArmA(a) => Branch::ArmA(a.map(visitor)),
            Branch::ArmB(b) => Branch::ArmB(b.map(visitor)),
        }
    }
}

impl<A, B> VisitLen for BranchModel<A, B>
where
    A: VisitLen,
    B: VisitLen,
{
    fn len(&self) -> usize {
        match self {
            BranchModel::ArmA { value, .. } => value.len(),
            BranchModel::ArmB { value, .. } => value.len(),
        }
    }
}

impl<A, B, V> Visit<V> for BranchModel<A, B>
where
    A: Visit<V>,
    B: Visit<V>,
{
    fn visit(&self, visitor: &mut V) -> Result<()> {
        match self {
            BranchModel::ArmA { value, .. } => value.visit(visitor),
            BranchModel::ArmB { value, .. } => value.visit(visitor),
        }
    }
}

impl<A, B, V> VisitMut<V> for BranchModel<A, B>
where
    A: VisitMut<V>,
    B: VisitMut<V>,
{
    fn visit_mut(&mut self, visitor: &mut V) -> Result<()> {
        match self {
            BranchModel::ArmA { value, .. } => value.visit_mut(visitor),
            BranchModel::ArmB { value, .. } => value.visit_mut(visitor),
        }
    }
}

impl<A, B, X, Y> UpdateWith<Branch<X, Y>> for BranchModel<A, B>
where
    A: UpdateWith<X>,
    B: UpdateWith<Y>,
{
    fn create_with(updater: Branch<X, Y>) -> Self {
        match updater {
            Branch::ArmA(a) => ArmA {
                value: A::create_with(a),
                other: None,
            },
            Branch::ArmB(b) => ArmB {
                value: B::create_with(b),
                other: None,
            },
        }
    }

    fn update_with(&mut self, updater: Branch<X, Y>, equality_matters: bool) -> bool {
        match (self, updater) {
            (ArmA { value, .. }, Branch::ArmA(v)) => {
                equality_matters & value.update_with(v, equality_matters)
            }
            (ArmB { value, .. }, Branch::ArmB(v)) => {
                equality_matters & value.update_with(v, equality_matters)
            }
            (this @ ArmA { .. }, Branch::ArmB(v)) => {
                take_mut::take(this, |this| {
                    let ArmA { value, other } = this
                    else {
                        unreachable!()
                    };

                    ArmB {
                        value: update_option(other, v),
                        other: Some(value),
                    }
                });
                false
            }
            (this @ ArmB { .. }, Branch::ArmA(v)) => {
                take_mut::take(this, |this| {
                    let ArmB { value, other } = this
                    else {
                        unreachable!()
                    };

                    ArmA {
                        value: update_option(other, v),
                        other: Some(value),
                    }
                });
                false
            }
        }
    }
}

fn update_option<T, U>(optioned: Option<T>, updater: U) -> T
where
    T: UpdateWith<U>,
{
    match optioned {
        Some(mut v) => {
            v.update_with(updater, false);
            v
        }
        None => T::create_with(updater),
    }
}

impl<A, B> SpecificUpdate for Branch<A, B>
where
    A: SpecificUpdate,
    B: SpecificUpdate,
    A::UpdateTo: UpdateWith<A>,
    B::UpdateTo: UpdateWith<B>,
{
    type UpdateTo = BranchModel<A::UpdateTo, B::UpdateTo>;
}
