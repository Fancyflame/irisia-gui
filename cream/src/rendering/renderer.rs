use anyhow::Result;
use skia_safe::Surface;

use crate::{
    event::Event,
    primary::Vec2,
    structure::{
        element::{ElementHandle, RcHandle},
        Element,
    },
};

use super::{children_list::ChildrenList, event_register::BubbleEventRegister};

pub struct Renderer {
    layers: Vec<ElementHandle>, // Final rendering order
    tree_builder: ChildrenList,
    event_register: BubbleEventRegister,
    surface: Surface,
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Self {
        let surface = Surface::new_raster_n32_premul((width, height)).expect("no surface!");

        Renderer {
            layers: Vec::new(),
            tree_builder: ChildrenList::new(),
            event_register: BubbleEventRegister::new(),
            surface,
        }
    }

    pub fn construct<'a, E: Element>(&'a mut self, application: RcHandle<E>) {
        self.layers.clear();
        self.event_register.clear();
        self.tree_builder.expand_tree(application);

        for (i, x) in self.tree_builder.iter_by_render_order().enumerate() {
            *x.borrow_mut().service_mut().renderer_index_mut() = Some(i);
            self.layers.push(x.clone());

            for ev in x.borrow_mut().service().event_target().events() {
                self.event_register.register_unchecked(*ev, i);
            }
        }
    }

    pub fn emit_event<E: Event>(&self, ev: E, args: &E::Arg, point: Vec2) {
        self.event_register.call(ev, args, point, &self.layers);
    }

    pub fn render(&mut self) -> Result<()> {
        let canvas = self.surface.canvas();
        for x in self.layers.iter() {
            x.borrow_mut().render(canvas)?;
        }
        Ok(())
    }
}
