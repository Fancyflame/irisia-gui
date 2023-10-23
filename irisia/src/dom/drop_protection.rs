use crate::{style::StyleContainer, Element};

use super::{ChildNodes, RcElementModel};

use std::ops::Deref;

pub struct DropProtection<El, Sty, Sc>(pub RcElementModel<El, Sty, Sc>)
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: ChildNodes + 'static;

impl<El, Sty, Sc> Deref for DropProtection<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: ChildNodes + 'static,
{
    type Target = RcElementModel<El, Sty, Sc>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<El, Sty, Sc> Drop for DropProtection<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: ChildNodes + 'static,
{
    fn drop(&mut self) {
        self.0.set_abandoned();
    }
}
