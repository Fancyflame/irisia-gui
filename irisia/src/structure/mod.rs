use anyhow::anyhow;

use crate::{
    application::event_comp::IncomingPointerEvent,
    el_model::{EMCreateCtx, ElementModel},
    element::Render,
    primitive::Region,
    ElementInterfaces, Result,
};

pub use self::{
    child_box::ChildBox, cond::conditional, pat_match::pat_match, repeat::repeat, single::single,
};

mod chain;
mod child_box;
mod cond;
mod pat_match;
mod repeat;
mod single;

pub trait VisitBy<Cp>: 'static {
    fn visit<V>(&self, v: &mut V) -> Result<()>
    where
        V: Visitor<Cp>;

    fn visit_mut<V>(&mut self, v: &mut V) -> Result<()>
    where
        V: VisitorMut<Cp>;
}

type PropsFn<'a, Cp> = &'a mut dyn FnMut(&Cp);
type LayoutFn<'a, Cp> = &'a mut dyn FnMut(&Cp) -> Option<Region>;

pub trait RenderMultiple<T>: 'static {
    fn render_all(&mut self, args: Render) -> Result<()>;
    fn props_all(&self, f: PropsFn<T>);
    fn layout_all(&mut self, f: LayoutFn<T>) -> Result<()>;
    fn emit_event_all(&mut self, ipe: &IncomingPointerEvent) -> bool;
    fn len(&self) -> usize;
}

pub trait Visitor<Cp> {
    fn visit<El>(&mut self, em: &ElementModel<El, Cp>) -> Result<()>
    where
        El: ElementInterfaces;
}

pub trait VisitorMut<Cp> {
    fn visit_mut<El>(&mut self, em: &mut ElementModel<El, Cp>) -> Result<()>
    where
        El: ElementInterfaces;
}

impl<T, Cp> RenderMultiple<Cp> for T
where
    T: VisitBy<Cp>,
    Cp: 'static,
{
    fn render_all(&mut self, args: Render) -> Result<()> {
        struct Vis<'a>(Render<'a>);

        impl<Cp: 'static> VisitorMut<Cp> for Vis<'_> {
            fn visit_mut<El>(&mut self, em: &mut ElementModel<El, Cp>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                em.render(self.0)
            }
        }

        self.visit_mut(&mut Vis(args))
    }

    fn props_all(&self, f: PropsFn<Cp>) {
        struct Vis<'a, Cp>(PropsFn<'a, Cp>);

        impl<Cp> Visitor<Cp> for Vis<'_, Cp> {
            fn visit<El>(&mut self, em: &ElementModel<El, Cp>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                (self.0)(&em.child_props);
                Ok(())
            }
        }

        self.visit(&mut Vis(f)).unwrap()
    }

    fn layout_all(&mut self, f: LayoutFn<Cp>) -> Result<()> {
        struct Vis<'a, Cp>(LayoutFn<'a, Cp>);

        impl<Cp> VisitorMut<Cp> for Vis<'_, Cp> {
            fn visit_mut<El>(&mut self, em: &mut ElementModel<El, Cp>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                let option = (self.0)(&em.child_props);
                match option {
                    Some(region) => {
                        em.set_draw_region(region);
                        Ok(())
                    }
                    None => Err(anyhow!("layouter is exhausted")),
                }
            }
        }

        self.visit_mut(&mut Vis(f))
    }

    fn emit_event_all(&mut self, ipe: &IncomingPointerEvent) -> bool {
        struct Vis<'a> {
            children_entered: bool,
            ipe: &'a IncomingPointerEvent<'a>,
        }

        impl<Cp> VisitorMut<Cp> for Vis<'_> {
            fn visit_mut<El>(&mut self, em: &mut ElementModel<El, Cp>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                self.children_entered |= em.on_pointer_event(self.ipe);
                Ok(())
            }
        }

        let mut ee = Vis {
            children_entered: false,
            ipe,
        };

        self.visit_mut(&mut ee).unwrap();
        ee.children_entered
    }

    fn len(&self) -> usize {
        struct Vis(usize);

        impl<Cp> Visitor<Cp> for Vis {
            fn visit<El>(&mut self, _: &ElementModel<El, Cp>) -> Result<()>
            where
                El: ElementInterfaces,
            {
                self.0 += 1;
                Ok(())
            }
        }

        let mut visitor = Vis(0);
        self.visit(&mut visitor).unwrap();
        visitor.0
    }
}

pub trait StructureCreate<Cp> {
    type Target: VisitBy<Cp>;

    fn create(self, ctx: &EMCreateCtx) -> Self::Target;
}

impl<Cp, F, R> StructureCreate<Cp> for F
where
    F: FnOnce(&EMCreateCtx) -> R,
    R: VisitBy<Cp>,
{
    type Target = R;

    fn create(self, ctx: &EMCreateCtx) -> Self::Target {
        self(ctx)
    }
}
