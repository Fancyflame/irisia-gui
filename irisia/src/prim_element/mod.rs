use std::{any::Any, cell::RefCell, rc::Rc, time::Duration};

use block::RenderBlock;
use irisia_backend::skia_safe::{Canvas, Region as SkRegion};
use rect::RenderRect;
use text::RenderText;

use crate::{application::content::GlobalContent, model2::Model, primitive::Region};

pub mod block;
pub mod rect;
pub mod text;

type Handle<T> = Rc<RefCell<T>>;

#[derive(Clone)]
pub struct EMCreateCtx {
    pub(crate) global_content: Rc<GlobalContent>,
}

pub trait RenderTree: Any {
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
        let draw_rect = draw_region.ceil_to_irect();
        if let Some(dirty_region) = self.dirty_region {
            if dirty_region.quick_reject_rect(draw_rect) {
                return false;
            }
            if !dirty_region.intersects_rect(draw_rect) {
                return false;
            }
        }
        true
    }
}

#[derive(Clone)]
pub enum Element {
    Block(Handle<RenderBlock>),
    Rect(Handle<RenderRect>),
    Text(Handle<RenderText>),
}

impl RenderTree for Element {
    fn render(&mut self, args: RenderArgs, draw_region: Region) {
        match self {
            Self::Block(el) => el.borrow_mut().render(args, draw_region),
            Self::Rect(el) => el.borrow_mut().render(args, draw_region),
            Self::Text(el) => el.borrow_mut().render(args, draw_region),
        }
    }
}

pub trait GetElement {
    fn get_element(&self) -> Element;
}

impl<T> Model for T
where
    T: GetElement + 'static,
{
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        f(self.get_element())
    }
}

struct Common {
    ctx: EMCreateCtx,
    prev_redraw_region: Option<Region>,
}
