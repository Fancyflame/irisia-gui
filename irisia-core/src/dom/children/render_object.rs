use std::{any::Any, time::Duration};

use anyhow::anyhow;

use crate::{
    application::event_comp::NewPointerEvent,
    dom::{layer::LayerRebuilder, ElementModel},
    element::Element,
    primitive::Region,
    structure::{slot::SlotModel, ControlFlow, VisitMut, Visitor, VisitorMut},
    style::StyleContainer,
    Result, StyleReader,
};

pub trait RenderObject: 'static {
    fn render(
        &mut self,
        lr: &mut LayerRebuilder,
        iter: &mut dyn Iterator<Item = Region>,
        interval: Duration,
    ) -> Result<()>;

    fn emit_event(&mut self, npe: &NewPointerEvent) -> bool;

    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> RenderObject for T
where
    T: for<'a, 'lr> VisitMut<RenderHelper<'a, 'lr>>
        + for<'a, 'root> VisitMut<EmitEventHelper<'a, 'root>>
        + 'static,
{
    fn render(
        &mut self,
        lr: &mut LayerRebuilder,
        iter: &mut dyn Iterator<Item = Region>,
        interval: Duration,
    ) -> Result<()> {
        let mut rh = RenderHelper {
            lr,
            iter,
            interval,
            result: Ok(()),
        };
        self.visit_mut(&mut rh);
        rh.result
    }

    fn emit_event(&mut self, npe: &NewPointerEvent) -> bool {
        let mut logical_entered = false;
        let mut eeh = EmitEventHelper {
            children_entered: &mut logical_entered,
            npe,
        };
        self.visit_mut(&mut eeh);
        logical_entered
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

struct RenderHelper<'a, 'lr> {
    lr: &'a mut LayerRebuilder<'lr>,
    iter: &'a mut dyn Iterator<Item = Region>,
    interval: Duration,
    result: Result<()>,
}

impl<El, Sty, Cc> VisitorMut<ElementModel<El, Sty, Cc>> for RenderHelper<'_, '_>
where
    El: Element,
    Cc: RenderObject,
{
    fn visit_mut(&mut self, data: &mut ElementModel<El, Sty, Cc>, control: &mut ControlFlow) {
        let Some(region) = self.iter.next()
        else {
            self.result=Err(anyhow!("regions in iterator is not much enough"));
            control.set_exit();
            return;
        };

        self.result = data.render(self.lr, self.interval);
        if self.result.is_err() {
            control.set_exit();
        }
    }
}

pub struct Peeker<'a, Sr>(&'a mut dyn FnMut(Sr));

impl<El, Sty, Cc, Sr> Visitor<ElementModel<El, Sty, Cc>> for Peeker<'_, Sr>
where
    El: Element,
    Sty: StyleContainer,
    Sr: StyleReader,
{
    fn visit(&mut self, data: &ElementModel<El, Sty, Cc>, _: &mut crate::structure::ControlFlow) {
        (self.0)(data.styles.read());
    }
}

struct EmitEventHelper<'a, 'root> {
    npe: &'a NewPointerEvent<'root>,
    children_entered: &'a mut bool,
}

impl<El, Sty, Cc> VisitorMut<ElementModel<El, Sty, Cc>> for EmitEventHelper<'_, '_>
where
    El: Element,
    Cc: RenderObject,
{
    fn visit_mut(&mut self, data: &mut ElementModel<El, Sty, Cc>, _: &mut ControlFlow) {
        *self.children_entered |= data.emit_event(self.npe);
    }
}

impl<T> RenderObject for SlotModel<T>
where
    T: RenderObject,
{
    fn render(
        &mut self,
        lr: &mut crate::dom::layer::LayerRebuilder,
        iter: &mut dyn Iterator<Item = crate::primitive::Region>,
        interval: std::time::Duration,
    ) -> crate::Result<()> {
        self.0.render(lr, iter, interval)
    }

    fn emit_event(&mut self, npe: &crate::application::event_comp::NewPointerEvent) -> bool {
        self.0.emit_event(npe)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
