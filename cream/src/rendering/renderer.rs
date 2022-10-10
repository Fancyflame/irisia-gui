use std::any::Any;

use skia_safe::{Canvas, Surface};

use crate::map_rc::MapRc;

use super::layer::LayerTrait;

pub struct Renderer {
    children_stack: Vec<MapRc<dyn Any>>,
    layers: Vec<MapRc<dyn LayerTrait>>,
    surface: Surface,
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Self {
        let surface = Surface::new_raster_n32_premul((width, height)).expect("no surface!");

        Renderer {
            children_stack: Vec::new(),
            layers: Vec::new(),
            surface,
        }
    }

    pub fn canvas(&mut self) -> &mut Canvas {
        self.surface.canvas()
    }
}
