use std::time::Duration;

use irisia_backend::skia_safe::Canvas;

use crate::{
    application::{
        event_comp::NewPointerEvent,
        redraw_scheduler::{RedrawObject, RedrawScheduler},
    },
    element::{ChildrenSetter, Element},
    primitive::Region,
    Result,
};

use self::{
    children::RenderMultiple,
    data_structure::maybe_shared::MaybeShared,
    layer::{LayerCompositer, LayerRebuilder},
};

pub use self::{data_structure::ElementHandle, update::add_one};
pub(crate) use self::{data_structure::ElementModel, update::EMUpdateContent};

pub(crate) mod children;
mod data_structure;
pub(crate) mod layer;
pub mod pub_handle;
mod render;
pub(crate) mod update;

impl<El, Sty, Sc> ElementModel<El, Sty, Sc> {
    pub fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>
    where
        El: Element,
    {
        match &mut self.shared {
            MaybeShared::Unique(unique) => unique.redraw(lr, interval),
            MaybeShared::Shared(shared) => {
                let canvas = lr.new_layer(shared.clone())?;
                shared.redraw(canvas, interval)
            }
        }
    }

    pub fn render_as_root(&self, canvas: &mut Canvas, interval: Duration) -> Result<()>
    where
        El: Element,
    {
        match &self.shared {
            MaybeShared::Shared(shared) => shared.redraw(canvas, interval),
            MaybeShared::Unique(_) => {
                unreachable!("expected self to have independent layer")
            }
        }
    }

    pub fn layout(&mut self, draw_region: Region)
    where
        El: Element,
        Sc: RenderMultiple + 'static,
    {
        let mut shared = self.shared.borrow_mut();
        shared.draw_region = draw_region;
        self.pub_shared.el_write_clean().layout(
            draw_region,
            &self.slot_cache,
            ChildrenSetter::new(
                &mut shared.expanded_children,
                &self.pub_shared.global(),
                *self.pub_shared.dep_layer_id.lock().unwrap(),
            ),
        )
    }

    /// returns whether this element is logically entered
    pub fn emit_event(&mut self, npe: &NewPointerEvent) -> bool {
        let mut shared = self.shared.borrow_mut();

        let Some(children_box) = &mut shared.expanded_children
        else {
            return false;
        };

        let children_logically_entered = children_box.as_render_multiple().emit_event(npe);
        self.event_mgr
            .update_and_emit(npe, shared.interact_region, children_logically_entered)
    }

    pub fn styles(&self) -> &Sty {
        &self.styles
    }

    fn update_independent_layer(&mut self, sch: &mut RedrawScheduler)
    where
        El: Element,
    {
        let indep_layer_locked = self
            .pub_shared
            .lock_independent_layer
            .load(std::sync::atomic::Ordering::Relaxed);

        match (&self.shared, indep_layer_locked) {
            (MaybeShared::Shared(_), false) => {
                let mut dep_layer_id = self.pub_shared.dep_layer_id.lock().unwrap();
                sch.del(*dep_layer_id);
                *dep_layer_id = self.shared.borrow().parent_layer_id;
                assert!(self.shared.try_to_unique());
            }

            (MaybeShared::Unique(_), true) => {
                self.shared.to_shared(LayerCompositer::new());

                let MaybeShared::Shared(shared) = &self.shared
                else {
                    unreachable!()
                };

                let indep_layer_id = sch.reg(shared.clone());

                let mut dep_layer_id = self.pub_shared.dep_layer_id.lock().unwrap();
                *dep_layer_id = indep_layer_id;
            }

            _ => {}
        }
    }
}
