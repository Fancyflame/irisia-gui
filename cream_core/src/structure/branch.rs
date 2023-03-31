use crate::{element::RenderContent, style::reader::StyleReader, Result};

use super::{RenderingNode, VisitIter};

#[derive(Default)]
pub struct BranchCache<T, U> {
    arm1: T,
    arm2: U,
}

pub enum Branch<T, U> {
    Arm1(T),
    Arm2(U),
}

impl<T, U> RenderingNode for Branch<T, U>
where
    T: RenderingNode,
    U: RenderingNode,
{
    type Cache = BranchCache<<T as RenderingNode>::Cache, <U as RenderingNode>::Cache>;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: RenderContent) {
        match self {
            Branch::Arm1(a) => a.prepare_for_rendering(&mut cache.arm1, content),
            Branch::Arm2(a) => a.prepare_for_rendering(&mut cache.arm2, content),
        }
    }
    fn element_count(&self) -> usize {
        match self {
            Branch::Arm1(a) => a.element_count(),
            Branch::Arm2(a) => a.element_count(),
        }
    }

    fn finish<S, F>(
        self,
        cache: &mut Self::Cache,
        content: RenderContent,
        map: &mut F,
    ) -> crate::Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<crate::primary::Region>,
        S: crate::style::reader::StyleReader,
    {
        match self {
            Branch::Arm1(a) => a.finish(&mut cache.arm1, content, map),
            Branch::Arm2(a) => a.finish(&mut cache.arm2, content, map),
        }
    }
}

impl<T, U, Prop> VisitIter<Prop> for Branch<T, U>
where
    T: VisitIter<Prop>,
    U: VisitIter<Prop>,
{
    type VisitIter<'a, S> =
        BranchIter<T::VisitIter<'a, S>, U::VisitIter<'a, S>>
        where
            S:StyleReader,
            Self: 'a;

    fn visit_iter<S>(&self) -> Self::VisitIter<'_, S>
    where
        S: StyleReader,
    {
        match self {
            Branch::Arm1(a) => BranchIter::Arm1(a.visit_iter()),
            Branch::Arm2(a) => BranchIter::Arm2(a.visit_iter()),
        }
    }
}

pub enum BranchIter<T, U> {
    Arm1(T),
    Arm2(U),
}

impl<T, U, I> Iterator for BranchIter<T, U>
where
    T: Iterator<Item = I>,
    U: Iterator<Item = I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BranchIter::Arm1(i) => i.next(),
            BranchIter::Arm2(i) => i.next(),
        }
    }
}
