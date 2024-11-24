pub struct NeedInit;

pub trait AsEmptyProps {
    type AsEmpty;
    fn empty_props() -> Self::AsEmpty;
}
