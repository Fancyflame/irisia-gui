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
    type StyleIter<'b, S> = <T as Node>::StyleIter<'b, S>
    where
        Self: 'b;

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader,
    {
        self.slot.node.style_iter()
    }

    fn finish_iter<'b, I>(self, _: &mut (), iter: I) -> Result<()>
    where
        I: Iterator<Item = WildRenderContent<'b>>,
    {
        self.slot.render(iter)
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
    pub fn style_iter<S>(&self) -> T::StyleIter<'_, S>
    where
        S: StyleReader,
    {
        self.node.style_iter()
    }

    pub fn render<'b, I>(self, iter: I) -> Result<()>
    where
        I: Iterator<Item = WildRenderContent<'b>>,
    {
        self.node.finish_iter(self.cache, iter)
    }
}
