use std::{any::Any, time::Duration};

use anyhow::anyhow;

use crate::{
    application::event_comp::IncomingPointerEvent,
    dom::layer::LayerRebuilder,
    primitive::Region,
    structure::{VisitBy, VisitOn},
    Result, StyleReader,
};

pub trait ChildNodes: 'static {
    fn render_raw(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;
    fn peek_styles_raw(&self, f: TypeElimatedSrGroup);
    fn len_raw(&self) -> usize;
    fn layout_raw(&self, iter: TypeElimatedLayouter) -> Result<()>;
    fn emit_event_raw(&self, ipe: &IncomingPointerEvent) -> bool;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> ChildNodes for T
where
    T: VisitBy + 'static,
{
    fn render_raw<'a, 'lr>(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
        self.visit_by(&mut RenderHelper { lr, interval })
    }

    fn peek_styles_raw(&self, f: TypeElimatedSrGroup) {
        self.visit_by(&mut PeekStyles { reader: f }).unwrap();
    }

    fn len_raw(&self) -> usize {
        VisitBy::len(self)
    }

    fn layout_raw(&self, layouter: TypeElimatedLayouter) -> Result<()> {
        self.visit_by(&mut LayoutHelper { layouter })
    }

    fn emit_event_raw(&self, ipe: &IncomingPointerEvent) -> bool {
        let mut eeh = EmitEventHelper {
            children_entered: false,
            ipe,
        };
        let _ = self.visit_by(&mut eeh);
        eeh.children_entered
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct RenderHelper<'a, 'lr> {
    lr: &'a mut LayerRebuilder<'lr>,
    interval: Duration,
}

impl VisitOn for RenderHelper<'_, '_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        data.build_layers(self.lr, self.interval)
    }
}

struct LayoutHelper<'a> {
    layouter: TypeElimatedLayouter<'a>,
}

impl VisitOn for LayoutHelper<'_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        let region = (self.layouter)(&data.in_cell.borrow().styles);
        match region {
            Some(region) => {
                data.set_draw_region(region);
                Ok(())
            }
            None => Err(anyhow!("unexpected end of layouter")),
        }
    }
}

struct PeekStyles<'a> {
    reader: TypeElimatedSrGroup<'a>,
}

impl VisitOn for PeekStyles<'_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        (self.reader)(&data.in_cell.borrow().styles);
        Ok(())
    }
}

struct EmitEventHelper<'a, 'root> {
    ipe: &'a IncomingPointerEvent<'root>,
    children_entered: bool,
}

impl VisitOn for EmitEventHelper<'_, '_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        self.children_entered |= data.emit_event(self.ipe);
        Ok(())
    }
}
