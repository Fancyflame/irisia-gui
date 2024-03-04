use std::time::Duration;

use irisia_backend::skia_safe::Canvas;

use crate::dom::layer::LayerRebuilder;

pub struct RenderElement<'a, 'lr> {
    pub(crate) lr: &'a mut LayerRebuilder<'lr>,
    pub(crate) interval: Duration,
}

impl<'a, 'lr> RenderElement<'a, 'lr> {
    pub(crate) fn new(lr: &'a mut LayerRebuilder<'lr>, interval: Duration) -> Self {
        RenderElement { lr, interval }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn canvas(&mut self) -> &Canvas {
        self.lr.draw_in_place()
    }
}
