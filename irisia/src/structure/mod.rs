use std::time::Duration;

use anyhow::anyhow;

use crate::{
    application::event_comp::IncomingPointerEvent,
    el_model::{layer::LayerRebuilder, EMCreateCtx, RenderOn, SharedEM},
    primitive::Region,
    style::ReadStyle,
    ElementInterfaces, Result,
};

pub use self::{child_box::ChildBox, repeat::repeat, select::branch, single::single};

mod chain;
mod child_box;
mod repeat;
mod select;
mod single;

pub trait VisitBy: 'static {
    fn visit<V>(&self, v: &mut V) -> Result<()>
    where
        V: Visitor;

    fn len(&self) -> usize;
}

pub trait RenderMultiple: 'static {
    fn render(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;

    fn peek_styles(&self, f: &mut dyn FnMut(&dyn ReadStyle));

    fn layout(&self, f: &mut dyn FnMut(&dyn ReadStyle) -> Option<Region>) -> Result<()>;

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
    fn render(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
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
                    RenderOn::ParentLayer(_) => em.el.borrow_mut().render(self.lr, self.interval),
                }
            }
        }

        self.visit(&mut Render { lr, interval })
    }

    fn peek_styles(&self, f: &mut dyn FnMut(&dyn ReadStyle)) {
        struct PeekStyles<F>(F);

        impl<F> Visitor for PeekStyles<F>
        where
            F: FnMut(&dyn ReadStyle),
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

    fn layout(&self, f: &mut dyn FnMut(&dyn ReadStyle) -> Option<Region>) -> Result<()> {
        struct Layout<F>(F);

        impl<F> Visitor for Layout<F>
        where
            F: FnMut(&dyn ReadStyle) -> Option<Region>,
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
        VisitBy::len(self)
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
