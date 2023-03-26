use crate::Result;

use super::Node;

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

impl<T, U> Node for Branch<T, U>
where
    T: Node,
    U: Node,
{
    type Cache = BranchCache<<T as Node>::Cache, <U as Node>::Cache>;
    type Iter<'a, S> =
        BranchIter<<T as Node>::Iter<'a, S>, <U as Node>::Iter<'a, S>>
        where
            Self: 'a;

    fn style_iter<S>(&self) -> Self::Iter<'_, S>
    where
        S: crate::style::reader::StyleReader,
    {
        match self {
            Branch::Arm1(a) => BranchIter::Arm1(a.style_iter()),
            Branch::Arm2(a) => BranchIter::Arm2(a.style_iter()),
        }
    }

    fn __finish_iter<S, F>(
        self,
        cache: &mut Self::Cache,
        content: crate::element::render_content::WildRenderContent,
        map: &mut F,
    ) -> crate::Result<()>
    where
        F: FnMut(S, Option<crate::primary::Region>) -> Result<crate::primary::Region>,
        S: crate::style::reader::StyleReader,
    {
        match self {
            Branch::Arm1(a) => a.__finish_iter(&mut cache.arm1, content, map),
            Branch::Arm2(a) => a.__finish_iter(&mut cache.arm2, content, map),
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
