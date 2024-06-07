use std::time::Duration;

use super::{RenderMultiple, StructureCreate};
use crate::{
    application::event_comp::IncomingPointerEvent, el_model::layer::LayerRebuilder,
    primitive::Region, structure::EMCreateCtx, style::ReadStyle, Result,
};

pub struct ChildBox(Box<dyn RenderMultiple>);

impl ChildBox {
    pub fn new<T>(updater: T, ctx: &EMCreateCtx) -> Self
    where
        T: StructureCreate,
    {
        ChildBox(Box::new(updater.create(ctx)))
    }

    pub fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
        self.0.render(lr, interval)
    }

    pub fn peek_styles<F>(&self, mut f: F)
    where
        F: FnMut(&dyn ReadStyle),
    {
        self.0.peek_styles(&mut f)
    }

    pub fn layout<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(&dyn ReadStyle) -> Option<Region>,
    {
        self.0.layout(&mut f)
    }

    pub fn emit_event(&self, ipe: &IncomingPointerEvent) -> bool {
        self.0.emit_event(ipe)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}
