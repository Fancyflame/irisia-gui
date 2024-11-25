pub struct NeedInit;

pub trait AsEmptyProps {
    type AsEmpty;
    fn empty_props() -> Self::AsEmpty;
}

impl AsEmptyProps for () {
    type AsEmpty = ();
    fn empty_props() -> Self::AsEmpty {
        ()
    }
}
