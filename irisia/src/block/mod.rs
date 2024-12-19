use std::time::Duration;

use irisia_backend::skia_safe::{Canvas, Region as SkRegion};

use crate::{primitive::Region, Result};

pub mod rect;
pub mod text;

pub trait RenderUnit {
    fn render(&mut self, args: RenderArgs, draw_region: Region);
}

#[derive(Clone, Copy)]
pub struct RenderArgs<'a> {
    pub canvas: &'a Canvas,
    pub interval: Duration,
    pub dirty_region: Option<&'a SkRegion>,
}

impl RenderArgs<'_> {
    pub fn needs_redraw(&self, draw_region: Region) -> bool {
        if let Some(dr) = self.dirty_region {
            if !dr.intersects_rect(draw_region.ceil_to_irect()) {
                return false;
            }
        }
        true
    }
}
