use std::time::Duration;

use anyhow::anyhow;
use irisia_backend::skia_safe::Canvas;

use crate::{
    dom::{children::RenderMultiple, layer::LayerRebuilder},
    Result,
};

pub struct RenderElement<'a, 'lr> {
    lr: &'a mut LayerRebuilder<'lr>,
    children: Option<&'a mut dyn RenderMultiple>,
    interval: Duration,
}

impl<'a, 'lr> RenderElement<'a, 'lr> {
    pub(crate) fn new(
        lr: &'a mut LayerRebuilder<'lr>,
        children: &'a mut dyn RenderMultiple,
        interval: Duration,
    ) -> Self {
        RenderElement {
            lr,
            children: Some(children),
            interval,
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn render_children(&mut self) -> Result<()> {
        match self.children.take() {
            Some(c) => c.render(self.lr, self.interval),
            None => Err(anyhow!("children cannot be rendered twice")),
        }
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        self.lr.draw_in_place()
    }
}
