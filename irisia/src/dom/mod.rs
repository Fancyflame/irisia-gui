use std::{
    rc::{Rc, Weak},
    time::Duration,
};

use anyhow::anyhow;
use irisia_backend::skia_safe::Canvas;

use crate::{
    application::{event_comp::NewPointerEvent, redraw_scheduler::RedrawObject},
    element::{Element, RenderElement},
    primitive::Region,
    style::StyleContainer,
    Result,
};

use self::{
    data_structure::InsideRefCell,
    layer::{LayerCompositer, LayerRebuilder},
};

pub(crate) use self::{
    children::RenderMultiple, drop_protection::DropProtection, update::EMUpdateContent,
};
pub use self::{data_structure::ElementModel, update::one_child};

pub(crate) mod children;
mod data_structure;
mod drop_protection;
pub(crate) mod layer;
pub mod pub_handle;
pub(crate) mod update;

pub type RcElementModel<El, Sty, Sc> = Rc<data_structure::ElementModel<El, Sty, Sc>>;

impl<El, Sty, Sc> ElementModel<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: RenderMultiple + 'static,
{
    pub(crate) fn build_layers(
        self: &Rc<Self>,
        lr: &mut LayerRebuilder,
        interval: Duration,
    ) -> Result<()> {
        let mut in_cell_ref = self.in_cell.borrow_mut();
        let in_cell = &mut *in_cell_ref;

        // update independent later status
        if !self.acquire_independent_layer.take() && in_cell.parent_layer.is_some() {
            in_cell.indep_layer = None;
        } else if in_cell.indep_layer.is_none() {
            in_cell.indep_layer = Some(LayerCompositer::new())
        }

        match &in_cell.indep_layer {
            None => self.el_write_clean().render(
                self,
                RenderElement::new(
                    lr,
                    in_cell
                        .expanded_children
                        .as_mut()
                        .map(|cb| cb.as_render_multiple()),
                    interval,
                ),
            ),
            Some(il) => lr.new_layer(il.clone()),
        }
    }

    pub(crate) fn set_draw_region(self: &Rc<Self>, region: Region) {
        if region == self.draw_region() {
            return;
        }
        self.draw_region.set(region);
        self.el_write_clean().draw_region_changed(self, region);
        self.set_dirty();
    }

    pub(crate) fn composite(&self, canvas: &mut Canvas) -> Result<()> {
        let in_cell = self.in_cell.borrow();
        match &in_cell.indep_layer {
            Some(il) => {
                debug_assert!(
                    in_cell.parent_layer.is_none(),
                    "illegal to call `composite` on non-root element"
                );
                il.borrow().composite(canvas)
            }
            None => panic_on_debug("cannot call `composite` on elements have no independent layer"),
        }
    }

    /// returns whether this element is logically entered
    pub fn emit_event(&self, npe: &NewPointerEvent) -> bool {
        let mut in_cell = self.in_cell.borrow_mut();

        let children_logically_entered = match &mut in_cell.expanded_children {
            Some(children_box) => children_box.as_render_multiple().emit_event(npe),
            None => false,
        };

        in_cell.event_mgr.update_and_emit(
            npe,
            self.interact_region.take(),
            children_logically_entered,
        )
    }

    fn get_children_layer(&self, in_cell: &InsideRefCell<Sty>) -> Weak<dyn RedrawObject>
    where
        Sty: 'static,
        Sc: 'static,
    {
        match in_cell.indep_layer {
            Some(_) => self.this.clone() as _,
            None => match &in_cell.parent_layer {
                Some(pl) => pl.clone(),
                None => unreachable!("root element did not initialize independent layer"),
            },
        }
    }

    fn set_abandoned(self: &Rc<Self>)
    where
        Sty: 'static,
        Sc: 'static,
    {
        let this = self.clone();
        tokio::task::spawn_local(async move {
            this.el_alive.set(false);
            this.el.write().await.take();
        });
    }
}

fn panic_on_debug(msg: &str) -> Result<()> {
    if cfg!(debug_assertions) {
        panic!("inner error: {}", msg);
    } else {
        Err(anyhow!("{}", msg))
    }
}

impl<El, Sty, Sc> RedrawObject for ElementModel<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: RenderMultiple + 'static,
{
    fn redraw(&self, canvas: &mut Canvas, interval: Duration) -> Result<()> {
        let mut in_cell_ref = self.in_cell.borrow_mut();
        let in_cell = &mut *in_cell_ref;

        let mut il = match &in_cell.indep_layer {
            Some(il) => il.borrow_mut(),
            None => {
                return panic_on_debug("this element model is expected to have independent layer");
            }
        };

        let mut rebuilder = il.rebuild(canvas);
        self.el_write_clean().render(
            &self.this.upgrade().unwrap(),
            RenderElement::new(
                &mut rebuilder,
                in_cell
                    .expanded_children
                    .as_mut()
                    .map(|cb| cb.as_render_multiple()),
                interval,
            ),
        )
    }
}
