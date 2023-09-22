use crate::{style::StyleContainer, Element};

use super::{RcElementModel, RenderMultiple};

use std::ops::Deref;

pub(crate) struct DropProtection<El, Sty, Sc>(pub RcElementModel<El, Sty, Sc>)
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: RenderMultiple + 'static;

impl<El, Sty, Sc> Deref for DropProtection<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: RenderMultiple + 'static,
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
    Sc: RenderMultiple + 'static,
{
    fn drop(&mut self) {
        self.0.set_no_longer_use();
    }
}
