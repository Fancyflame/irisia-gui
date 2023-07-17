use anyhow::anyhow;

use crate::{primitive::Region, style::StyleContainer, Result};

use super::{activate::Layouter, VisitItem};

pub struct LayoutOnce(Option<Region>);

impl LayoutOnce {
    pub fn new(region: Region) -> Self {
        Self(Some(region))
    }
}

impl<El> Layouter<El> for LayoutOnce {
    fn layout(&mut self, _: VisitItem<El, impl StyleContainer>) -> Result<Region> {
        self.0
            .take()
            .ok_or_else(|| anyhow!("at most 1 element can be rendered"))
    }
}
