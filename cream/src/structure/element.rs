use std::{any::Any, cell::RefCell, rc::Rc};

use anyhow::Result;
use skia_safe::Canvas;

use crate::{primary::Vec2, rendering::children_list::ExpandTree};

use super::elem_svc::ElemService;

pub type RcHandle<T> = Rc<RefCell<T>>;
pub type ElementHandle = RcHandle<dyn Element>;

pub trait Element: Any {
    fn render(&mut self, canvas: &mut Canvas) -> Result<()>;
    fn service_mut(&mut self) -> &mut ElemService;
    fn service(&self) -> &ElemService;
    fn request_size(&self, max_size: Vec2) -> Vec2 {
        max_size
    }
    fn expand_tree(&self, expand: ExpandTree);
}

// A: (Rc<dyn Watchable<T0>>, Rc<dyn Watchable<T1>>, ...)
pub trait CreateElement<A>: Element + Sized {
    fn create(args: A) -> Result<Self>;
}
