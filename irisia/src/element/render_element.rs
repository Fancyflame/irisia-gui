use std::time::Duration;

use anyhow::anyhow;
use irisia_backend::skia_safe::Canvas;

use crate::{
    application::redraw_scheduler::IndepLayerRegister,
    dom::{children::RenderMultiple, layer::LayerRebuilder},
    primitive::Region,
    Result,
};

pub struct RenderElement<'a, 'lr> {
    lr: &'a mut LayerRebuilder<'lr>,
    reg: &'a mut IndepLayerRegister,
    children: Option<&'a mut dyn RenderMultiple>,
    interact_region: &'a mut Option<Region>,
    interval: Duration,
}

impl<'a, 'lr> RenderElement<'a, 'lr> {
    pub(crate) fn new(
        lr: &'a mut LayerRebuilder<'lr>,
        reg: &'a mut IndepLayerRegister,
        children: &'a mut dyn RenderMultiple,
        interact_region: &'a mut Option<Region>,
        interval: Duration,
    ) -> Self {
        RenderElement {
            lr,
            reg,
            children: Some(children),
            interact_region,
            interval,
        }
    }

    pub fn render_children(&mut self) -> Result<&mut Self> {
        match self.children.take() {
            Some(c) => {
                c.render(self.lr, self.reg, self.interval)?;
                Ok(self)
            }
            None => Err(anyhow!("children cannot be rendered twice")),
        }
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        self.lr.draw_in_place()
    }

    pub fn set_interact_region(&mut self, region: Region) -> &mut Self {
        *self.interact_region = Some(region);
        self
    }

    pub fn clear_interact_region(&mut self) -> &mut Self {
        self.interact_region.take();
        self
    }
}
