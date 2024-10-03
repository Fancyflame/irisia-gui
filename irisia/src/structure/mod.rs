use std::time::Duration;

use anyhow::anyhow;

use crate::{
    application::event_comp::IncomingPointerEvent,
    el_model::{layer::LayerRebuilder, EMCreateCtx, RenderOn, SharedEM},
    primitive::Region,
    style::StyleFn,
    ElementInterfaces, Result,
};

pub use self::{
    child_box::ChildBox,
    cond::conditional,
    pat_match::pat_match,
    repeat::{repeat, RepeatMutator},
    single::single,
};

mod chain;
mod child_box;
mod cond;
mod pat_match;
mod repeat;
mod single;

pub trait VisitBy: 'static {
    fn visit<V>(&self, v: &mut V) -> Result<()>
    where
        V: Visitor;
}

pub trait RenderMultiple: 'static {
    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;

    fn peek_styles(&self, f: &mut dyn FnMut(&dyn StyleFn));

    fn layout(&mut self, f: &mut dyn FnMut(&dyn StyleFn) -> Option<Region>) -> Result<()>;

    fn emit_event(&self, ipe: &IncomingPointerEvent) -> bool;

    fn len(&self) -> usize;
}

pub trait Visitor {
    fn visit<El>(&mut self, em: &SharedEM<El>) -> Result<()>
    where
        El: ElementInterfaces;
}

impl<T> RenderMultiple for T
where
    T: VisitBy,
{
    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
        struct Render<'a, 'b> {
            lr: &'a mut LayerRebuilder<'b>,
            interval: Duration,
        }

        impl Visitor for Render<'_, '_> {
            fn visit<El>(&mut self, em: &SharedEM<El>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                em.shared.redraw_signal_sent.set(false);
                match &em.shared.render_on {
                    RenderOn::NewLayer { layer, .. } => self.lr.append_layer(layer),
                    RenderOn::ParentLayer(_) => em.monitoring_render(self.lr, self.interval),
                }
            }
        }

        self.visit(&mut Render { lr, interval })
    }

    fn peek_styles(&self, f: &mut dyn FnMut(&dyn StyleFn)) {
        struct PeekStyles<F>(F);

        impl<F> Visitor for PeekStyles<F>
        where
            F: FnMut(&dyn StyleFn),
        {
            fn visit<El>(&mut self, em: &SharedEM<El>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                (self.0)(&em.shared.styles);
                Ok(())
            }
        }

        self.visit(&mut PeekStyles(f)).unwrap()
    }

    fn layout(&mut self, f: &mut dyn FnMut(&dyn StyleFn) -> Option<Region>) -> Result<()> {
        struct Layout<F>(F);

        impl<F> Visitor for Layout<F>
        where
            F: FnMut(&dyn StyleFn) -> Option<Region>,
        {
            fn visit<El>(&mut self, em: &SharedEM<El>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                let option = (self.0)(&em.shared.styles);
                match option {
                    Some(region) => {
                        let old_region = em.shared.draw_region.get();

                        if region != old_region {
                            em.set_draw_region(region);
                            em.request_redraw();
                        }

                        Ok(())
                    }
                    None => Err(anyhow!("layouter is exhausted")),
                }
            }
        }

        self.visit(&mut Layout(f))
    }

    fn emit_event(&self, ipe: &IncomingPointerEvent) -> bool {
        struct EmitEvent<'a> {
            children_entered: bool,
            ipe: &'a IncomingPointerEvent<'a>,
        }

        impl Visitor for EmitEvent<'_> {
            fn visit<El>(&mut self, em: &SharedEM<El>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                self.children_entered |= em.on_pointer_event(self.ipe);
                Ok(())
            }
        }

        let mut ee = EmitEvent {
            children_entered: false,
            ipe,
        };

        self.visit(&mut ee).unwrap();
        ee.children_entered
    }

    fn len(&self) -> usize {
        struct VisitLength(usize);

        impl Visitor for VisitLength {
            fn visit<El>(&mut self, _: &SharedEM<El>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                self.0 += 1;
                Ok(())
            }
        }

        let mut visitor = VisitLength(0);
        self.visit(&mut visitor).unwrap();
        visitor.0
    }
}

pub trait StructureCreate {
    type Target: VisitBy;

    fn create(self, ctx: &EMCreateCtx) -> Self::Target;
}

impl<F, R> StructureCreate for F
where
    F: FnOnce(&EMCreateCtx) -> R,
    R: VisitBy,
{
    type Target = R;
    fn create(self, ctx: &EMCreateCtx) -> Self::Target {
        self(ctx)
    }
}
