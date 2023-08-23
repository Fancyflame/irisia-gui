use std::time::Duration;

use irisia_backend::skia_safe::Canvas;

use crate::{
    application::{
        event_comp::NewPointerEvent,
        redraw_scheduler::{IndepLayerRegister, RedrawObject},
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
    pub(crate) fn render(
        &mut self,
        lr: &mut LayerRebuilder,
        reg: &mut IndepLayerRegister,
        interval: Duration,
    ) -> Result<()>
    where
        El: Element,
    {
        self.update_independent_layer(reg);

        match &mut self.shared {
            MaybeShared::Unique(unique) => unique.redraw(lr, reg, interval),
            MaybeShared::Shared(shared) => {
                let canvas = lr.new_layer(shared.clone())?;
                shared.redraw(canvas, reg, interval)
            }
        }
    }

    pub(crate) fn render_as_root(
        &self,
        canvas: &mut Canvas,
        reg: &mut IndepLayerRegister,
        interval: Duration,
    ) -> Result<()>
    where
        El: Element,
    {
        match &self.shared {
            MaybeShared::Shared(shared) => shared.redraw(canvas, reg, interval),
            MaybeShared::Unique(_) => {
                unreachable!("expected self to have independent layer")
            }
        }
    }

    pub(crate) fn layout(&mut self, draw_region: Region)
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
                self.pub_shared.layer_info.read().unwrap().render_layer_id(),
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

    fn update_independent_layer(&mut self, reg: &mut IndepLayerRegister)
    where
        El: Element,
    {
        let mut layer_info = self.pub_shared.layer_info.write().unwrap();

        match (&self.shared, layer_info.acquire_independent_layer) {
            (MaybeShared::Shared(_), false) => {
                reg.del(
                    layer_info
                        .indep_layer_id
                        .take()
                        .expect("independent layer id expected to be exists"),
                );

                assert!(self.shared.try_to_unique());
            }

            (MaybeShared::Unique(_), true) => {
                self.shared.to_shared(LayerCompositer::new());

                let MaybeShared::Shared(shared) = &self.shared
                else {
                    unreachable!()
                };

                debug_assert!(layer_info.indep_layer_id.is_none());
                layer_info.indep_layer_id = Some(reg.reg(shared.clone()));
            }

            _ => {}
        }
    }
}
