use irisia_backend::skia_safe::Color;
use layout::{DefaultLayouter, LayoutChildren};
use rect::DrawRRect;

use crate::{
    hook::Signal,
    primitive::{
        Length, Point,
        length::{LengthStandard, LengthStandardGlobalPart},
    },
};

use super::{
    Common, EMCreateCtx, Element, EmitEventArgs, EventCallback, RenderTree, Size, SpaceConstraint,
    read_or_default, redraw_guard::RedrawGuard,
};

pub use layout::BlockLayout;

pub mod layout;
mod rect;

#[derive(Clone, Copy)]
pub struct BlockStyle {
    pub width: Length,
    pub height: Length,
    pub margin: f32,
    pub background: Color,
    pub border_width: f32,
    pub border_color: Color,
    pub border_radius: [f32; 4],
}

impl BlockStyle {
    pub const DEFAULT: Self = Self {
        width: Length::Auto,
        height: Length::Auto,
        margin: 0.0,
        background: Color::TRANSPARENT,
        border_width: 0.0,
        border_color: Color::BLACK,
        border_radius: [0.0; 4],
    };
}

impl Default for BlockStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

struct Child {
    element: Element,
    location: Point,
    cached_layout: Option<(Size<SpaceConstraint>, Size<f32>)>,
}

pub struct RenderBlock {
    layouter: Option<Signal<dyn BlockLayout>>,
    style: Option<Signal<BlockStyle>>,
    cached_background_rect: Option<DrawRRect>,
    children: ElementList,
    needs_check_children_sizes: bool,
    common: Common,
}

pub struct InitRenderBlock<'a> {
    pub layouter: Option<Signal<dyn BlockLayout>>,
    pub style: Option<Signal<BlockStyle>>,
    pub children: ElementList,
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
        self.cached_background_rect.take();
        self.common.request_relayout();
    }

    pub fn style_updated(&mut self) {
        // TODO: 需要判断width和height是否改变
        self.cached_background_rect.take();
        self.common.request_relayout();
        self.common.request_redraw();
    }

    pub fn update_children(&mut self) -> RedrawGuard<ElementList> {
        self.common.cached_layout.take();
        self.children.0.clear();
        RedrawGuard::new(&mut self.children, &mut self.common)
    }

    fn layout_tree(&mut self, constraint: Size<SpaceConstraint>, force_compute: bool) -> Size<f32> {
        let length_standard_gp = self.common.ctx.global_content.length_standard();
        let layout_fn = |constraint: Size<SpaceConstraint>| {
            let wh_style = match self.style.as_ref() {
                Some(style) => {
                    let style = style.read();
                    Size {
                        width: style.width,
                        height: style.height,
                    }
                }
                None => Size::default(),
            };

            let new_constraint = Size {
                width: constraint_single_axis_styled(
                    constraint.width,
                    wh_style.width,
                    &length_standard_gp,
                ),
                height: constraint_single_axis_styled(
                    constraint.height,
                    wh_style.height,
                    &length_standard_gp,
                ),
            };

            let this_size = read_or_default(&self.layouter, &DefaultLayouter)
                .compute_layout(LayoutChildren::new(&mut self.children.0), new_constraint);

            if self
                .children
                .0
                .iter()
                .any(|child| child.cached_layout.is_none())
            {
                panic!("there still some elements not being layouted");
            }

            self.cached_background_rect.take();

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

fn constraint_single_axis_styled(
    input_constraint: SpaceConstraint,
    length: Length,
    ls: &LengthStandardGlobalPart,
) -> SpaceConstraint {
    let Some(except) = length.to_resolved(LengthStandard {
        global: *ls,
        parent_axis_len: input_constraint.as_parent_size().unwrap_or(0.0),
    }) else {
        return input_constraint;
    };

    SpaceConstraint::Exact(match input_constraint {
        SpaceConstraint::Available(available) => available.min(except),
        SpaceConstraint::Exact(value) => value,
        SpaceConstraint::MinContent | SpaceConstraint::MaxContent => except,
    })
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
            .get_or_insert_with(|| {
                DrawRRect::new(
                    &read_or_default(&self.style, &BlockStyle::DEFAULT),
                    draw_region,
                )
            })
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
