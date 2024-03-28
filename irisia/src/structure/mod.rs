use std::time::Duration;

use anyhow::anyhow;

use crate::{
    application::event_comp::IncomingPointerEvent,
    dom::{layer::LayerRebuilder, EMCreateCtx},
    primitive::Region,
    style::StyleSource,
    Element, Result,
};

pub use self::{
    chain::Chain,
    once::{Once, OnceUpdater},
    repeat::{Repeat, RepeatUpdater},
    select::{Select, SelectState},
};

mod chain;
mod empty;
mod once;
mod repeat;
mod select;

pub trait VisitBy {
    fn iter(&self) -> impl Iterator<Item = &dyn Element>;

    fn visit_mut(&mut self, f: impl FnMut(&mut dyn Element) -> Result<()>) -> Result<()>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
        self.visit_mut(|el| el.render(lr, interval))
    }

    fn peek_styles(&self) -> impl Iterator<Item = &dyn StyleSource> {
        self.iter().map(Element::get_style)
    }

    fn layout(&mut self, mut f: impl FnMut(&dyn StyleSource) -> Option<Region>) -> Result<()> {
        self.visit_mut(|el| {
            let region = f(el.get_style());
            match region {
                Some(r) => {
                    el.set_draw_region(r);
                    Ok(())
                }
                None => Err(anyhow!("layouter is exhausted")),
            }
        })
    }

    fn emit_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        let mut children_entered = false;
        let _ = self.visit_mut(|el| {
            children_entered |= el.on_pointer_event(ipe);
            Ok(())
        });
        children_entered
    }
}

pub trait StructureUpdater {
    type Target: VisitBy;

    fn update(self, this: &mut Self::Target, ctx: &EMCreateCtx);
    fn create(self, ctx: &EMCreateCtx) -> Self::Target;
}
