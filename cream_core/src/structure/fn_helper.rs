use crate::element::RenderContent;

pub trait FuncOnce<'a, Input> {
    type Output: Iterator<Item = RenderContent<'a>>;
    fn provide(self, input: Input) -> Self::Output;
}

impl<'a, F, Input, Output> FuncOnce<'a, Input> for F
where
    F: FnOnce(Input) -> Output,
    Output: Iterator<Item = RenderContent<'a>>,
{
    type Output = Output;
    fn provide(self, input: Input) -> Output {
        self(input)
    }
}
