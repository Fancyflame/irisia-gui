use super::{Style, StyleContainer};

#[derive(Clone)]
pub struct Chain<Bsc, Ext> {
    basic: Bsc,
    extend: Ext,
}

impl<B, E> StyleContainer for Chain<B, E>
where
    B: StyleContainer,
    E: StyleContainer,
{
    fn get_style<T: Style>(&self) -> Option<T> {
        self.basic.get_style().or_else(|| self.extend.get_style())
    }
}

impl<B, E> Chain<B, E>
where
    B: StyleContainer,
    E: StyleContainer,
{
    pub fn new(basic: B, extend: E) -> Self {
        Chain { basic, extend }
    }
}
