use std::{any::Any, rc::Rc, time::Duration};

use anyhow::anyhow;

use crate::{
    application::{event_comp::NewPointerEvent, redraw_scheduler::RedrawObject},
    dom::{data_structure::Context, layer::LayerRebuilder, DropProtection},
    element::{Element, GlobalContent},
    primitive::Region,
    structure::{VisitBy, VisitLen, Visitor},
    style::{style_box::InsideStyleBox, StyleContainer},
    Result,
};

pub trait ChildNodes: 'static {
    fn render(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;

    fn peek_styles(&self, f: &mut dyn FnMut(&dyn InsideStyleBox));

    fn len(&self) -> usize;

    fn layout(&self, f: &mut dyn FnMut(&dyn InsideStyleBox) -> Option<Region>) -> Result<()>;

    fn emit_event(&self, npe: &NewPointerEvent) -> bool;

    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> ChildNodes for T
where
    T: for<'a, 'lr> VisitBy<RenderHelper<'a, 'lr>>
        + for<'a, 'root> VisitBy<EmitEventHelper<'a, 'root>>
        + for<'a> VisitBy<LayoutHelper<'a>>
        + for<'a> VisitBy<PeekStyles<'a>>
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
    Sc: ChildNodes,
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
    Sc: ChildNodes,
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
    Sc: ChildNodes,
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
    Sc: ChildNodes,
{
    fn visit(&mut self, data: &DropProtection<El, Sty, Sc>) -> Result<()> {
        *self.children_entered |= data.emit_event(self.npe);
        Ok(())
    }
}

struct AttachHelper<'a> {
    parent_layer: Option<&'a Rc<dyn RedrawObject>>,
    global_content: Rc<GlobalContent>,
}

impl<El, Sty, Sc> Visitor<DropProtection<El, Sty, Sc>> for AttachHelper<'_>
where
    El: Element,
    Sty: StyleContainer,
    Sc: ChildNodes,
{
    fn visit(&mut self, data: &DropProtection<El, Sty, Sc>) -> Result<()> {
        let mut borrowed = data.in_cell.borrow_mut();
        match &mut borrowed.context {
            Context::None => {
                *borrowed = Context::Attached {
                    global_content: self.global_content.clone(),
                    parent_layer: self.parent_layer,
                };
                Ok(())
            }

            Context::Attached {
                global_content,
                parent_layer,
            } => {
                if !Rc::ptr_eq(global_content, &self.global_content) {
                    return Err(anyhow!(
                        "cannot attach element node in another context to this"
                    ));
                }
                *parent_layer = self.parent_layer.map(Rc::downgrade);
                Ok(())
            }

            Context::Destroyed => Err(anyhow!("cannot update context to an abondoned element")),
        }
    }
}
