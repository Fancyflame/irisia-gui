use std::{sync::Arc, time::Duration};

use crate::{
    application::{
        content::GlobalContent,
        event_comp::{NewPointerEvent, NodeEventMgr},
    },
    element::{render_element::RenderElement, ComputeSize, Element},
    primitive::Region,
    Result,
};

use self::{
    children::ChildrenBox,
    layer::{LayerRebuilder, SharedLayerCompositer},
};

pub use update::add_one;

pub(crate) mod children;
pub(crate) mod layer;
pub mod peek_children;
pub mod update;

pub struct ElementModel<El, Sty> {
    element: El,
    styles: Sty,
    global_content: Arc<GlobalContent>,
    independent_layer: Option<SharedLayerCompositer>,
    event_mgr: NodeEventMgr,
    interact_region: Option<Region>,
    computed_size: ComputeSize,
    expanded_children: ChildrenBox,
}

impl<El, Sty> ElementModel<El, Sty>
where
    El: Element,
{
    pub fn render(
        &mut self,
        lr: &mut LayerRebuilder,
        draw_region: Region,
        interval: Duration,
    ) -> Result<()> {
        let mut render = |lr: &mut LayerRebuilder<'_>| {
            self.element.render(
                draw_region,
                RenderElement::new(
                    lr,
                    self.expanded_children.as_render_object(),
                    &mut self.interact_region,
                    interval,
                ),
                interval,
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

    /// returns whether this element is logically entered
    pub(crate) fn emit_event(&mut self, npe: &NewPointerEvent) -> bool {
        let children_logically_entered = self.expanded_children.as_render_object().emit_event(npe);
        self.event_mgr
            .update_and_emit(npe, self.interact_region, children_logically_entered)
    }
}
