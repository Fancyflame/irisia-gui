use std::{cell::RefCell, time::Duration};

use irisia_backend::skia_safe::Canvas;

use crate::{
    application::redraw_scheduler::{IndepLayerRegister, RedrawObject},
    element::RenderElement,
    Element, Result,
};

use super::{
    children::ChildrenBox,
    data_structure::{LayerSharedPart, RcIndepLayer},
    layer::{CustomLayer, LayerRebuilder},
};

impl<El> LayerSharedPart<El>
where
    El: Element,
{
    pub(super) fn redraw(
        &mut self,
        lr: &mut LayerRebuilder,
        reg: &mut IndepLayerRegister,
        interval: Duration,
    ) -> Result<()> {
        self.pub_shared.el_write_clean().render(
            RenderElement::new(
                lr,
                reg,
                unwrap_children(&mut self.expanded_children).as_render_multiple(),
                &mut self.interact_region,
                interval,
            ),
            interval,
            self.draw_region,
        )
    }
}

fn unwrap_children(cb: &mut Option<ChildrenBox>) -> &mut ChildrenBox {
    cb.as_mut()
        .unwrap_or_else(|| unreachable!("children not initialized"))
}

impl<El> RedrawObject for RefCell<RcIndepLayer<El>>
where
    El: Element,
{
    fn redraw(
        &self,
        canvas: &mut Canvas,
        reg: &mut IndepLayerRegister,
        interval: Duration,
    ) -> Result<()> {
        let mut this = self.borrow_mut();
        let inner = &mut *this;
        let ret = inner
            .main
            .redraw(&mut inner.extra.rebuild(canvas), reg, interval);
        ret
    }
}

impl<El> CustomLayer for RefCell<RcIndepLayer<El>> {
    fn composite(&self, canvas: &mut Canvas) -> Result<()> {
        self.borrow().extra.composite(canvas)
    }
}
