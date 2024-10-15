use super::{RenderMultiple, StructureCreate, VisitBy};
use crate::{
    application::event_comp::IncomingPointerEvent, element::Render, primitive::Region,
    structure::EMCreateCtx, Result,
};

pub struct ChildBox<Cp>(Box<dyn RenderMultiple<Cp>>);

impl<Cp: 'static> ChildBox<Cp> {
    pub fn new<T>(updater: T, ctx: &EMCreateCtx) -> Self
    where
        T: StructureCreate,
        T::Target: VisitBy<Cp>,
    {
        ChildBox(Box::new(updater.create(ctx)))
    }

    pub fn render(&mut self, args: Render) -> Result<()> {
        self.0.render_all(args)
    }

    pub fn props<F>(&self, mut f: F)
    where
        F: FnMut(&Cp),
    {
        self.0.props_all(&mut f);
    }

    pub fn layout<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(&Cp) -> Option<Region>,
    {
        self.0.layout_all(&mut f)
    }

    pub fn emit_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.0.emit_event_all(ipe)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn check_mark_dirty(&mut self, dirty_region: Region) {
        self.0.check_mark_dirty_all(dirty_region);
    }
}
