use std::{any::Any, time::Duration};

use anyhow::anyhow;

use crate::{
    application::event_comp::NewPointerEvent,
    dom::{layer::LayerRebuilder, DropProtection},
    element::Element,
    primitive::Region,
    structure::{Visit, VisitLen, Visitor},
    style::{style_box::InsideStyleBox, StyleContainer},
    Result,
};

pub trait RenderMultiple: 'static {
    fn render(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;

    fn peek_styles(&self, f: &mut dyn FnMut(&dyn InsideStyleBox));

    fn len(&self) -> usize;

    fn layout(&self, f: &mut dyn FnMut(&dyn InsideStyleBox) -> Option<Region>) -> Result<()>;

    fn emit_event(&self, npe: &NewPointerEvent) -> bool;

    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> RenderMultiple for T
where
    T: for<'a, 'lr> Visit<RenderHelper<'a, 'lr>>
        + for<'a, 'root> Visit<EmitEventHelper<'a, 'root>>
        + for<'a> Visit<LayoutHelper<'a>>
        + for<'a> Visit<PeekStyles<'a>>
        + 'static,
{
    fn render(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
        self.visit(&mut RenderHelper { lr, interval })
    }

    fn peek_styles(&self, f: &mut dyn FnMut(&dyn InsideStyleBox)) {
        let _ = self.visit(&mut PeekStyles(f));
    }

    fn len(&self) -> usize {
        VisitLen::len(self)
    }

    fn layout(&self, f: &mut dyn FnMut(&dyn InsideStyleBox) -> Option<Region>) -> Result<()> {
        self.visit(&mut LayoutHelper(f))
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

impl<El, Sty, Sc> Visitor<DropProtection<El, Sty, Sc>> for RenderHelper<'_, '_>
where
    El: Element,
    Sty: StyleContainer,
    Sc: RenderMultiple,
{
    fn visit(&mut self, data: &DropProtection<El, Sty, Sc>) -> Result<()> {
        data.build_layers(self.lr, self.interval)
    }
}

struct LayoutHelper<'a>(&'a mut dyn FnMut(&dyn InsideStyleBox) -> Option<Region>);

impl<El, Sty, Sc> Visitor<DropProtection<El, Sty, Sc>> for LayoutHelper<'_>
where
    El: Element,
    Sty: StyleContainer,
    Sc: RenderMultiple,
{
    fn visit(&mut self, data: &DropProtection<El, Sty, Sc>) -> Result<()> {
        let region = (self.0)(&data.in_cell.borrow().styles);
        match region {
            Some(region) => {
                data.set_draw_region(region);
                Ok(())
            }
            None => Err(anyhow!("unexpected end of layouter")),
        }
    }
}

struct PeekStyles<'a>(&'a mut dyn FnMut(&dyn InsideStyleBox));

impl<El, Sty, Sc> Visitor<DropProtection<El, Sty, Sc>> for PeekStyles<'_>
where
    El: Element,
    Sty: StyleContainer,
    Sc: RenderMultiple,
{
    fn visit(&mut self, data: &DropProtection<El, Sty, Sc>) -> Result<()> {
        (self.0)(&data.in_cell.borrow().styles);
        Ok(())
    }
}

struct EmitEventHelper<'a, 'root> {
    npe: &'a NewPointerEvent<'root>,
    children_entered: &'a mut bool,
}

impl<El, Sty, Sc> Visitor<DropProtection<El, Sty, Sc>> for EmitEventHelper<'_, '_>
where
    El: Element,
    Sty: StyleContainer,
    Sc: RenderMultiple,
{
    fn visit(&mut self, data: &DropProtection<El, Sty, Sc>) -> Result<()> {
        *self.children_entered |= data.emit_event(self.npe);
        Ok(())
    }
}
