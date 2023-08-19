use std::{any::Any, time::Duration};

use anyhow::anyhow;

use crate::{
    application::event_comp::NewPointerEvent,
    dom::{layer::LayerRebuilder, ElementModel},
    element::Element,
    primitive::Region,
    structure::{slot::Slot, VisitMut, VisitorMut},
    Result,
};

pub trait RenderMultiple: 'static {
    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;

    fn layout(&mut self, iter: &mut dyn Iterator<Item = Region>) -> Result<()>;

    fn emit_event(&mut self, npe: &NewPointerEvent) -> bool;

    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> RenderMultiple for T
where
    T: for<'a, 'lr> VisitMut<RenderHelper<'a, 'lr>>
        + for<'a, 'root> VisitMut<EmitEventHelper<'a, 'root>>
        + for<'a> VisitMut<LayoutHelper<'a>>
        + 'static,
{
    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
        self.visit_mut(&mut RenderHelper { lr, interval })
    }

    fn layout(&mut self, iter: &mut dyn Iterator<Item = Region>) -> Result<()> {
        self.visit_mut(&mut LayoutHelper { iter })
    }

    fn emit_event(&mut self, npe: &NewPointerEvent) -> bool {
        let mut logical_entered = false;
        let mut eeh = EmitEventHelper {
            children_entered: &mut logical_entered,
            npe,
        };
        let _ = self.visit_mut(&mut eeh);
        logical_entered
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

struct RenderHelper<'a, 'lr> {
    lr: &'a mut LayerRebuilder<'lr>,
    interval: Duration,
}

impl<El, Sty, Cc> VisitorMut<ElementModel<El, Sty, Cc>> for RenderHelper<'_, '_>
where
    El: Element,
{
    fn visit_mut(&mut self, data: &mut ElementModel<El, Sty, Cc>) -> Result<()> {
        data.render(self.lr, self.interval)
    }
}

struct LayoutHelper<'a> {
    iter: &'a mut dyn Iterator<Item = Region>,
}

impl<El, Sty, Cc> VisitorMut<ElementModel<El, Sty, Cc>> for LayoutHelper<'_>
where
    El: Element,
    Cc: RenderMultiple,
{
    fn visit_mut(&mut self, data: &mut ElementModel<El, Sty, Cc>) -> Result<()> {
        match self.iter.next() {
            Some(region) => {
                data.layout(region);
                Ok(())
            }
            None => Err(anyhow!("regions in the iterator is not enough")),
        }
    }
}

struct EmitEventHelper<'a, 'root> {
    npe: &'a NewPointerEvent<'root>,
    children_entered: &'a mut bool,
}

impl<El, Sty, Cc> VisitorMut<ElementModel<El, Sty, Cc>> for EmitEventHelper<'_, '_>
where
    El: Element,
{
    fn visit_mut(&mut self, data: &mut ElementModel<El, Sty, Cc>) -> Result<()> {
        *self.children_entered |= data.emit_event(self.npe);
        Ok(())
    }
}

impl<T> RenderMultiple for Slot<T>
where
    T: RenderMultiple,
{
    fn render(
        &mut self,
        lr: &mut crate::dom::layer::LayerRebuilder,
        interval: std::time::Duration,
    ) -> crate::Result<()> {
        self.0.borrow_mut().render(lr, interval)
    }

    fn layout(&mut self, iter: &mut dyn Iterator<Item = Region>) -> Result<()> {
        self.0.borrow_mut().layout(iter)
    }

    fn emit_event(&mut self, npe: &crate::application::event_comp::NewPointerEvent) -> bool {
        self.0.borrow_mut().emit_event(npe)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
