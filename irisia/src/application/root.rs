use std::{
    cell::{RefCell, RefMut},
    rc::{Rc, Weak},
    time::Duration,
};

use irisia_backend::skia_safe::Canvas;

use crate::{
    dom::layer::{LayerCompositer, SharedLayerCompositer},
    Element, Result,
};

use super::redraw_scheduler::StandaloneRender;

pub(super) struct Root<El> {
    el: RefCell<El>,
    lc: SharedLayerCompositer,
}

impl<El> Root<El>
where
    El: Element,
{
    pub fn new<F>(el_creator: F) -> Rc<Self>
    where
        F: FnOnce(Weak<dyn StandaloneRender>) -> El,
    {
        Rc::new_cyclic(|weak| Root {
            el: RefCell::new(el_creator(weak.clone() as _)),
            lc: LayerCompositer::new(),
        })
    }

    pub fn el(&self) -> RefMut<El> {
        self.el.borrow_mut()
    }

    pub fn composite(&self, canvas: &Canvas) -> Result<()> {
        self.lc.borrow_mut().composite(canvas)
    }
}

impl<El> StandaloneRender for Root<El>
where
    El: Element,
{
    fn standalone_render(&self, canvas: &Canvas, interval: Duration) -> Result<()> {
        self.el
            .borrow_mut()
            .render(&mut LayerCompositer::rebuild(&self.lc, canvas), interval)
    }
}
