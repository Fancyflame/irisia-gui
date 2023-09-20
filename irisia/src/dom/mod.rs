use std::{rc::Rc, time::Duration};

use crate::{
    application::event_comp::NewPointerEvent,
    element::{Element, RenderElement},
    primitive::Region,
    style::StyleContainer,
    Result,
};

use self::layer::{LayerCompositer, LayerRebuilder};

pub(crate) use self::update::EMUpdateContent;
pub use self::{children::RenderMultiple, data_structure::ElementModel, update::add_one};

pub(crate) mod children;
mod data_structure;
pub(crate) mod layer;
pub mod pub_handle;
mod render;
pub(crate) mod update;

pub type RcElementModel<El, Sty, Sc> = Rc<data_structure::ElementModel<El, Sty, Sc>>;

impl<El, Sty, Sc> ElementModel<El, Sty, Sc>
where
    El: Element,
{
    pub(crate) fn render(&self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()> {
        let mut in_cell_ref = self.in_cell.borrow_mut();
        let in_cell = &mut *in_cell_ref;

        let mut r = |lr: &mut LayerRebuilder| {
            self.el_write_clean().render(RenderElement::new(
                lr,
                in_cell
                    .expanded_children
                    .as_mut()
                    .unwrap()
                    .as_render_multiple(),
                interval,
            ))
        };

        match &in_cell.indep_layer {
            None => r(lr),
            Some(il) => {
                let canvas = lr.new_layer(il.clone())?;
                r(&mut il.borrow_mut().rebuild(canvas))
            }
        }
    }

    pub(crate) fn set_draw_region(self: &Rc<Self>, region: Region)
    where
        Sty: StyleContainer,
        Sc: RenderMultiple,
    {
        self.draw_region.set(region);
        self.el_write_clean().draw_region_changed(self, region);
    }

    /// returns whether this element is logically entered
    pub fn emit_event(&self, npe: &NewPointerEvent) -> bool {
        let mut in_cell = self.in_cell.borrow_mut();

        let Some(children_box) = &in_cell.expanded_children
        else {
            unreachable!("children must be set");
        };

        let children_logically_entered = children_box.as_render_multiple().emit_event(npe);
        in_cell.event_mgr.update_and_emit(
            npe,
            self.interact_region.take(),
            children_logically_entered,
        )
    }

    fn update_independent_layer(&mut self) {
        let mut in_cell = self.in_cell.borrow_mut();
        let acq = self.acquire_independent_layer.take();

        if !acq {
            in_cell.indep_layer = None;
        } else if in_cell.indep_layer.is_none() {
            in_cell.indep_layer = Some(LayerCompositer::new())
        }
    }
}
