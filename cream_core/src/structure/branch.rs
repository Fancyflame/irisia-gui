use crate::{element::RenderContent, Result};

use super::RenderingNode;

#[derive(Default)]
pub struct BranchCache<T, U> {
    arm1: T,
    arm2: U,
}

pub enum Branch<T, U> {
    Arm1(T),
    Arm2(U),
}

pub enum BranchIter<T, U> {
    Arm1(T),
    Arm2(U),
}

impl<T, U> RenderingNode for Branch<T, U>
where
    T: RenderingNode,
    U: RenderingNode,
{
    type Cache = BranchCache<<T as RenderingNode>::Cache, <U as RenderingNode>::Cache>;
    type StyleIter<'a, S> =
        BranchIter<T::StyleIter<'a, S>, U::StyleIter<'a, S>>
        where
            Self: 'a;
    type RegionIter<'a> = BranchIter<T::RegionIter<'a>, U::RegionIter<'a>>
    where
        Self:'a;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: RenderContent) {
        match self {
            Branch::Arm1(a) => a.prepare_for_rendering(&mut cache.arm1, content),
            Branch::Arm2(a) => a.prepare_for_rendering(&mut cache.arm2, content),
        }
    }

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: crate::style::reader::StyleReader,
    {
        match self {
            Branch::Arm1(a) => BranchIter::Arm1(a.style_iter()),
            Branch::Arm2(a) => BranchIter::Arm2(a.style_iter()),
        }
    }

    fn region_iter(&self) -> Self::RegionIter<'_> {
        match self {
            Branch::Arm1(a) => BranchIter::Arm1(a.region_iter()),
            Branch::Arm2(a) => BranchIter::Arm2(a.region_iter()),
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
