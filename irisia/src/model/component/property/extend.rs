pub trait PropExtend<T> {
    type Output<Ext>;
    fn prop_extend<Ext>(self, f: impl FnOnce(T) -> Ext) -> Self::Output<Ext>;
}

impl<T> PropExtend<T> for T {
    type Output<Ext> = Ext;
    fn prop_extend<Ext>(self, f: impl FnOnce(T) -> Ext) -> Self::Output<Ext> {
        f(self)
    }
}

pub trait PropOwnedBy {
    type Owner;
}

pub struct ExtendHelper<Src, Value, Dst> {
    pub(crate) value: Value,
    pub(crate) updator: fn(Src, Value) -> Dst,
}

impl<Src, Value, Dst> ExtendHelper<Src, Value, Dst> {
    pub fn new(value: Value, updator: fn(Src, Value) -> Dst) -> Self {
        Self { value, updator }
    }

    pub fn apply<Ext>(self, target: Ext) -> Ext::Output<Dst>
    where
        Ext: PropExtend<Src>,
    {
        target.prop_extend(|src| (self.updator)(src, self.value))
    }
}
