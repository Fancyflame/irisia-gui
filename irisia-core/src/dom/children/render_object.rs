use std::{any::Any, time::Duration};

use anyhow::anyhow;

use crate::{
    application::event_comp::NewPointerEvent,
    dom::{layer::LayerRebuilder, ElementModel},
    element::Element,
    primitive::Region,
    structure::{ControlFlow, VisitMut, VisitorMut},
    Result,
};

pub trait RenderObject {
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

impl<El> VisitorMut<ElementModel<El>> for RenderHelper<'_, '_>
where
    El: Element,
{
    fn visit_mut(&mut self, data: &mut ElementModel<El>, control: &mut ControlFlow) {
        let Some(region) = self.iter.next()
        else {
            self.result=Err(anyhow!("regions in iterator is not much enough"));
            control.set_exit();
            return;
        };

        self.result = data.render(self.lr, region, self.interval);
        if self.result.is_err() {
            control.set_exit();
        }
    }
}

struct EmitEventHelper<'a, 'root> {
    npe: &'a NewPointerEvent<'root>,
    children_entered: &'a mut bool,
}

impl<El> VisitorMut<ElementModel<El>> for EmitEventHelper<'_, '_>
where
    El: Element,
{
    fn visit_mut(&mut self, data: &mut ElementModel<El>, _: &mut ControlFlow) {
        *self.children_entered |= data.emit_event(self.npe);
    }
}
