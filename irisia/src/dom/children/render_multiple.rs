use std::{any::Any, rc::Rc, time::Duration};

use anyhow::anyhow;

use crate::{
    application::{event_comp::NewPointerEvent, redraw_scheduler::IndepLayerRegister},
    dom::{layer::LayerRebuilder, RcElementModel},
    element::Element,
    primitive::Region,
    structure::{Visit, VisitMut, Visitor, VisitorMut},
    Result,
};

pub trait RenderMultiple: 'static {
    fn render(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;

    fn layout(&self, iter: &mut dyn Iterator<Item = Region>) -> Result<()>;

    fn emit_event(&self, npe: &NewPointerEvent) -> bool;

    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> RenderMultiple for T
where
    T: for<'a, 'lr> Visit<RenderHelper<'a, 'lr>>
        + for<'a, 'root> Visit<EmitEventHelper<'a, 'root>>
        + for<'a> Visit<LayoutHelper<'a>>
        + 'static,
{
    fn render(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
        self.visit(&mut RenderHelper { lr, interval })
    }

    fn layout(&self, iter: &mut dyn Iterator<Item = Region>) -> Result<()> {
        self.visit(&mut LayoutHelper { iter })
    }

    fn emit_event(&self, npe: &NewPointerEvent) -> bool {
        let mut logical_entered = false;
        let mut eeh = EmitEventHelper {
            children_entered: &mut logical_entered,
            npe,
        };
        let _ = self.visit(&mut eeh);
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

impl<El, Sty, Sc> Visitor<RcElementModel<El, Sty, Sc>> for RenderHelper<'_, '_>
where
    El: Element,
{
    fn visit(&mut self, data: &RcElementModel<El, Sty, Sc>) -> Result<()> {
        data.render(self.lr, self.interval)
    }
}

struct LayoutHelper<'a> {
    iter: &'a mut dyn Iterator<Item = Region>,
}

impl<El, Sty, Sc> Visitor<RcElementModel<El, Sty, Sc>> for LayoutHelper<'_>
where
    El: Element,
    Sc: RenderMultiple,
{
    fn visit(&mut self, data: &RcElementModel<El, Sty, Sc>) -> Result<()> {
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

impl<El, Sty, Sc> Visitor<RcElementModel<El, Sty, Sc>> for EmitEventHelper<'_, '_>
where
    El: Element,
{
    fn visit(&mut self, data: &RcElementModel<El, Sty, Sc>) -> Result<()> {
        *self.children_entered |= data.emit_event(self.npe);
        Ok(())
    }
}
