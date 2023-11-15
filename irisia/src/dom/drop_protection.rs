use crate::{style::StyleContainer, Element};

use super::RcElementModel;

use std::ops::Deref;

pub struct DropProtection<El, Sty>(pub RcElementModel<El, Sty>)
where
    El: Element,
    Sty: StyleContainer + 'static;

impl<El, Sty> Deref for DropProtection<El, Sty>
where
    El: Element,
    Sty: StyleContainer + 'static,
{
    type Target = RcElementModel<El, Sty>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<El, Sty> Drop for DropProtection<El, Sty>
where
    El: Element,
    Sty: StyleContainer + 'static,
{
    fn drop(&mut self) {
        self.0.set_abandoned();
    }
}
