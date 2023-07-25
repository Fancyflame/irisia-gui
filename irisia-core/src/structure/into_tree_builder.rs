use crate::{application::content::GlobalContent, element::SelfCache};

use super::{activate::Structure, TreeBuilder};

pub struct IntoTreeBuilder<'a, T: Structure> {
    pub(crate) cache: &'a mut SelfCache<T>,
    pub(crate) content: GlobalContent<'a>,
}

impl<'a, 'root, T: Structure> IntoTreeBuilder<'a, T> {
    pub fn into_builder(self, nodes: T, equality_matters: bool) -> TreeBuilder<'a, T::Activated> {
        TreeBuilder::new(nodes, self.cache, self.content, equality_matters)
    }
}
