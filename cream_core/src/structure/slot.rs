use super::*;

pub struct ApplySlot<'a, T>
where
    T: Node,
{
    slot: Slot<'a, T>,
}

impl<'a, T> ApplySlot<'a, T>
where
    T: Node,
{
    pub fn new(slot: Slot<'a, T>) -> Self {
        ApplySlot { slot }
    }
}

impl<'a, T> Node for ApplySlot<'a, T>
where
    T: Node,
{
    type Cache = ();
    type Iter<'b, S> = <T as Node>::Iter<'b, S>
    where
        Self: 'b;

    fn style_iter<S>(&self) -> Self::Iter<'_, S>
    where
        S: StyleReader,
    {
        self.slot.node.style_iter()
    }

    fn __finish_iter<S, F>(self, _: &mut (), content: WildRenderContent, map: &mut F) -> Result<()>
    where
        F: FnMut(S, Option<Region>) -> Result<Region>,
        S: StyleReader,
    {
        self.slot.finish(content, map)
    }
}

pub struct Slot<'a, T>
where
    T: Node,
{
    pub(crate) node: T,
    pub(crate) cache: &'a mut T::Cache,
}

impl<'a, T> Slot<'a, T>
where
    T: Node,
{
    pub fn style_iter<S>(&self) -> T::Iter<'_, S>
    where
        S: StyleReader,
    {
        self.node.style_iter()
    }

    fn finish<S, F>(self, content: WildRenderContent, map: &mut F) -> Result<()>
    where
        F: FnMut(S, Option<Region>) -> Result<Region>,
        S: StyleReader,
    {
        self.node.__finish_iter(self.cache, content, map)
    }
}
