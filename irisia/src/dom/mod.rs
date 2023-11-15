use std::{
    rc::{Rc, Weak},
    time::Duration,
};

use anyhow::anyhow;
use irisia_backend::skia_safe::Canvas;

use crate::{
    application::{event_comp::IncomingPointerEvent, redraw_scheduler::StandaloneRender},
    element::Element,
    event::standard::DrawRegionChanged,
    primitive::Region,
    style::StyleContainer,
    Result,
};

use self::{
    child_nodes::RenderElement,
    data_structure::{AttachedCtx, Context, InsideRefCell},
    layer::{LayerCompositer, LayerRebuilder},
};

pub use self::data_structure::ElementModel;
pub(crate) use self::{child_nodes::ChildNodes, drop_protection::DropProtection};

pub(crate) mod child_nodes;
mod data_structure;
mod drop_protection;
pub(crate) mod layer;
pub mod pub_handle;
pub(crate) mod update;

pub type RcElementModel<El, Sty> = Rc<data_structure::ElementModel<El, Sty>>;

impl<El, Sty> ElementModel<El, Sty>
where
    El: Element,
    Sty: StyleContainer + 'static,
{
    pub(crate) fn build_layers(
        self: &Rc<Self>,
        lr: &mut LayerRebuilder,
        interval: Duration,
    ) -> Result<()> {
        let mut in_cell_ref = self.in_cell.borrow_mut();
        let in_cell = &mut *in_cell_ref;

        // update independent later status
        if !self.acquire_independent_layer.get() && in_cell.parent_layer().is_some() {
            in_cell.indep_layer = None;
        } else if in_cell.indep_layer.is_none() {
            in_cell.indep_layer = Some(LayerCompositer::new())
        }

        match &in_cell.indep_layer {
            None => El::render(self, RenderElement::new(lr, interval)),
            Some(il) => lr.new_layer(il.clone()),
        }
    }

    pub(crate) fn set_draw_region(self: &Rc<Self>, region: Region) {
        if region == self.draw_region() {
            return;
        }
        self.draw_region.set(region);
        self.ed.emit_trusted(DrawRegionChanged { region });
    }

    pub(crate) fn composite_as_root(&self, canvas: &mut Canvas) -> Result<()> {
        let in_cell = self.in_cell.borrow();
        match &in_cell.indep_layer {
            Some(il) => {
                debug_assert!(
                    in_cell.parent_layer().is_none(),
                    "illegal to call `composite` on non-root element"
                );
                il.borrow().composite(canvas)
            }
            None => panic_on_debug("cannot call `composite` on elements have no independent layer"),
        }
    }

    /// returns whether this element is logically entered
    pub(crate) fn emit_event(&self, ipe: &IncomingPointerEvent) -> bool {
        let mut in_cell = self.in_cell.borrow_mut();

        let children_logically_entered = self.el_write_clean().slot().emit_event(ipe);

        in_cell.event_mgr.update_and_emit(
            ipe,
            self.interact_region.get(),
            children_logically_entered,
        )
    }

    fn get_children_layer(&self, in_cell: &InsideRefCell<Sty>) -> Weak<dyn StandaloneRender>
    where
        Sty: 'static,
    {
        match in_cell.indep_layer {
            Some(_) => self.this.clone() as _,
            None => match &in_cell.parent_layer() {
                Some(pl) => pl.clone(),
                None => unreachable!("root element did not initialize independent layer"),
            },
        }
    }

    fn set_abandoned(self: &Rc<Self>)
    where
        Sty: 'static,
    {
        let this = self.clone();
        tokio::task::spawn_local(async move {
            this.el.write().await.take();
            this.in_cell.borrow_mut().context = Context::Destroyed;
        });
    }
}

impl<Sty> InsideRefCell<Sty> {
    fn ctx(&self) -> Result<&AttachedCtx> {
        match &self.context {
            Context::None => Err(anyhow!("element have not attached to window yet")),
            Context::Attached(attached) => Ok(attached),
            Context::Destroyed => Err(anyhow!("element has been abandoned")),
        }
    }

    fn parent_layer(&self) -> &Option<Weak<dyn StandaloneRender>> {
        &self.ctx().unwrap().parent_layer
    }
}

fn panic_on_debug(msg: &str) -> Result<()> {
    if cfg!(debug_assertions) {
        panic!("inner error: {}", msg);
    } else {
        Err(anyhow!("{}", msg))
    }
}

impl<El, Sty> StandaloneRender for ElementModel<El, Sty>
where
    El: Element,
    Sty: StyleContainer + 'static,
{
    fn standalone_render(&self, canvas: &mut Canvas, interval: Duration) -> Result<()> {
        let mut in_cell_ref = self.in_cell.borrow_mut();
        let in_cell = &mut *in_cell_ref;

        let mut il = match &in_cell.indep_layer {
            Some(il) => il.borrow_mut(),
            None => {
                return panic_on_debug("this element model is expected to have independent layer");
            }
        };

        let mut rebuilder = il.rebuild(canvas);
        El::render(
            &self.this.upgrade().unwrap(),
            RenderElement::new(&mut rebuilder, interval),
        )
    }
}
