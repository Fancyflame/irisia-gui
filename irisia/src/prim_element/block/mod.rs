use irisia_backend::skia_safe::Color;
use layout::LayoutChildren;
use rect::DrawRRect;

use crate::{
    hook::Signal,
    primitive::{Point, Region},
};

use super::{
    redraw_guard::RedrawGuard, Common, EMCreateCtx, Element, EmitEventArgs, EventCallback,
    RenderTree, Size, SpaceConstraint,
};

pub use layout::BlockLayout;

pub mod layout;
mod rect;

pub type LayoutFn = fn(Point, &[Element], &mut Vec<Region>);

pub struct BlockStyle {
    pub margin: f32,
    pub background: Color,
    pub border_width: f32,
    pub border_color: Color,
    pub border_radius: [f32; 4],
}

impl Default for BlockStyle {
    fn default() -> Self {
        Self {
            margin: 0.0,
            background: Color::TRANSPARENT,
            border_width: 0.0,
            border_color: Color::BLACK,
            border_radius: [0.0; 4],
        }
    }
}

struct Child {
    element: Element,
    location: Point,
    cached_layout: Option<(Size<SpaceConstraint>, Size<f32>)>,
}

pub struct RenderBlock {
    layouter: Signal<dyn BlockLayout>,
    style: Signal<BlockStyle>,
    cached_background_rect: Option<DrawRRect>,
    children: ElementList,
    needs_check_children_sizes: bool,
    common: Common,
}

pub struct InitRenderBlock<'a> {
    pub style: Signal<BlockStyle>,
    pub children: ElementList,
    pub layouter: Signal<dyn BlockLayout>,
    pub event_callback: Option<EventCallback>,
    pub ctx: &'a EMCreateCtx,
}

impl RenderBlock {
    pub fn new(init: InitRenderBlock) -> Self {
        Self {
            layouter: init.layouter,
            style: init.style,
            cached_background_rect: None,
            children: init.children,
            needs_check_children_sizes: false,
            common: Common::new(init.event_callback, init.ctx),
        }
    }

    pub fn set_children_size_changed(&mut self) {
        self.needs_check_children_sizes = true;
        self.common.request_relayout();
    }

    pub fn layouter_updated(&mut self) {
        self.common.cached_layout.take();
        self.common.request_relayout();
    }

    pub fn style_updated(&mut self) {
        self.cached_background_rect = None;
        self.common.request_redraw();
    }

    pub fn update_children(&mut self) -> RedrawGuard<ElementList> {
        self.common.cached_layout.take();
        RedrawGuard::new(&mut self.children, &mut self.common)
    }

    fn layout_tree(&mut self, constraint: Size<SpaceConstraint>, force_compute: bool) -> Size<f32> {
        let layout_fn = |constraint| {
            let this_size = self
                .layouter
                .read()
                .compute_layout(LayoutChildren::new(&mut self.children.0), constraint);

            if self
                .children
                .0
                .iter()
                .any(|child| child.cached_layout.is_none())
            {
                panic!("there still some element not being layouted");
            }

            this_size
        };

        let result = self
            .common
            .use_cached_layout(constraint, force_compute, layout_fn);
        self.needs_check_children_sizes = false;
        result
    }

    fn recompute_and_check_child_sizes(&mut self) -> bool {
        for child in self.children.0.iter_mut() {
            let Some((constraint, old_size)) = child.cached_layout else {
                return true;
            };

            if child.element.layout(constraint) != old_size {
                return true;
            }
        }
        false
    }
}

impl RenderTree for RenderBlock {
    fn render(&mut self, args: super::RenderArgs, location: Point) {
        if self.needs_check_children_sizes || self.common.cached_layout.is_none() {
            panic!("must layout before render");
        }

        let draw_region = self.common.set_rendered(location);

        if !args.needs_redraw(draw_region) {
            return;
        }

        self.cached_background_rect
            .get_or_insert_with(|| DrawRRect::new(&*self.style.read(), draw_region))
            .draw(args.canvas);

        for child in self.children.0.iter_mut() {
            child.element.render(args, location + child.location);
        }
    }

    fn layout(&mut self, constraint: Size<SpaceConstraint>) -> Size<f32> {
        let force_compute = self.common.cached_layout.is_none()
            || (self.needs_check_children_sizes && self.recompute_and_check_child_sizes());

        self.layout_tree(constraint, force_compute)
    }

    fn emit_event(&mut self, mut args: EmitEventArgs) {
        for child in self.children.0.iter_mut().rev() {
            child.element.emit_event(args.reborrow());
        }

        self.common.use_callback(args);
    }

    fn set_callback(&mut self, callback: EventCallback) {
        self.common.event_callback = Some(callback);
    }
}

pub struct ElementList(Vec<Child>);

impl ElementList {
    pub fn new() -> Self {
        ElementList(Vec::new())
    }

    pub fn push(&mut self, el: Element) {
        self.0.push(Child {
            element: el,
            cached_layout: None,
            location: Point::ZERO,
        });
    }
}
