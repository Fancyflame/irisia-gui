use std::{sync::Arc, time::Duration};

use crate::{
    application::event_comp::{NewPointerEvent, NodeEventMgr},
    element::{render_element::RenderElement, ChildrenSetter, Element},
    primitive::Region,
    structure::slot::Slot,
    Result,
};

use self::{
    children::{ChildrenBox, RenderObject},
    layer::{LayerRebuilder, SharedLayerCompositer},
    shared::ElementHandle,
};

pub use update::add_one;

pub(crate) mod children;
pub(crate) mod layer;
pub mod shared;
pub mod update;

pub struct ElementModel<El, Sty, Cc> {
    styles: Sty,
    slot_cache: Slot<Cc>,
    independent_layer: Option<SharedLayerCompositer>,
    event_mgr: NodeEventMgr,
    expanded_children: Option<ChildrenBox>,
    interact_region: Option<Region>,
    draw_region: Region,
    shared_part: Arc<ElementHandle<El>>,
}

impl<El, Sty, Cc> ElementModel<El, Sty, Cc> {
    pub fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>
    where
        El: Element,
    {
        let mut render = |lr: &mut LayerRebuilder<'_>| {
            self.shared_part.el_mut().render(
                RenderElement::new(
                    lr,
                    unwrap_children(&mut self.expanded_children).as_render_object(),
                    &mut self.interact_region,
                    interval,
                ),
                interval,
                self.draw_region,
            )
        };

        match &self.independent_layer {
            Some(rc) => {
                let mut borrow_mut = rc.borrow_mut();
                let mut lr2 = lr.new_layer(&mut borrow_mut)?;
                render(&mut lr2)
            }
            None => render(lr),
        }
    }

    pub fn layout(&mut self, draw_region: Region, equality_matters: &mut bool)
    where
        El: Element,
        Cc: RenderObject + 'static,
    {
        self.draw_region = draw_region;
        self.shared_part.el_mut().layout(
            draw_region,
            &self.slot_cache,
            ChildrenSetter::new(
                &mut self.expanded_children,
                &self.shared_part.global(),
                equality_matters,
            ),
        )
    }

    /// returns whether this element is logically entered
    pub(crate) fn emit_event(&mut self, npe: &NewPointerEvent) -> bool {
        let Some(children_box) = &mut self.expanded_children
        else {
            return false;
        };

        let children_logically_entered = children_box.as_render_object().emit_event(npe);
        self.event_mgr
            .update_and_emit(npe, self.interact_region, children_logically_entered)
    }

    pub(crate) fn styles(&self) -> &Sty {
        &self.styles
    }
}

fn unwrap_children(cb: &mut Option<ChildrenBox>) -> &mut ChildrenBox {
    cb.as_mut()
        .unwrap_or_else(|| unreachable!("children not initialized"))
}
