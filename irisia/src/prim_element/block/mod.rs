use irisia_backend::skia_safe::Color;
use layout::{DefaultLayouter, LayoutChildren};
use rect::{DrawRRect, DrawRRectProps};

use crate::{
    hook::Signal,
    primitive::{Length, Point, corner::Corner, rect::Rect},
};

use super::{
    Common, EMCreateCtx, Element, EmitEventArgs, EventCallback, RenderArgs, RenderTree,
    RenderTreeExt, Size, WeakElement, layout::LayoutInput, read_or_default,
    redraw_guard::RedrawGuard,
};

pub use layout::BlockLayout;

pub mod layout;
mod rect;

#[derive(Clone, Copy)]
pub struct BlockStyle {
    pub width: Length,
    pub height: Length,
    pub margin: Rect<Length>,
    pub background: Color,
    pub border_width: Rect<Length>,
    pub border_color: Color,
    pub border_radius: Corner<f32>,
    pub padding: Rect<Length>,
    pub box_sizing: BoxSizing,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxSizing {
    #[default]
    ContentBox,
    BorderBox,
}

impl BlockStyle {
    pub const DEFAULT: Self = Self {
        width: Length::Auto,
        height: Length::Auto,
        margin: Rect::all(Length::Auto),
        background: Color::TRANSPARENT,
        border_width: Rect::all(Length::Auto),
        border_color: Color::BLACK,
        border_radius: Corner::all(0.0),
        padding: Rect::all(Length::Auto),
        box_sizing: BoxSizing::ContentBox,
    };
}

impl Default for BlockStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

struct Child<Cd> {
    element: Element,
    child_data: Cd,
}

pub struct RenderBlock<Cd> {
    layouter: Option<Signal<dyn BlockLayout<Cd>>>,
    style: Option<Signal<BlockStyle>>,
    cached_background_rect: Option<DrawRRect>,
    children: ElementList<Cd>,
    // prev_content_length_standard: Option<Size<LengthStandard>>,
    common: Common,
}

pub struct InitRenderBlock<'a, Cd = ()> {
    pub this: WeakElement,
    pub layouter: Option<Signal<dyn BlockLayout<Cd>>>,
    pub style: Option<Signal<BlockStyle>>,
    pub children: ElementList<Cd>,
    pub event_callback: Option<EventCallback>,
    pub ctx: &'a EMCreateCtx,
}

impl<Cd> RenderBlock<Cd> {
    pub fn new(init: InitRenderBlock<Cd>) -> Self {
        Self {
            layouter: init.layouter,
            style: init.style,
            cached_background_rect: None,
            children: init.children,
            common: Common::new(init.this, init.event_callback, init.ctx),
        }
    }

    pub fn layouter_updated(&mut self) {
        // self.cached_background_rect.take();
        self.common.request_reflow();
    }

    pub fn style_updated(&mut self) {
        // TODO: 需要判断width和height是否改变
        self.cached_background_rect.take();
        self.common.request_reflow();
        self.common.request_repaint();
    }

    pub fn update_children(&mut self) -> RedrawGuard<ElementList<Cd>> {
        self.common.request_reflow();
        self.children.0.clear();
        RedrawGuard::new(&mut self.children, &mut self.common)
    }
}

impl<Cd: 'static> RenderTree for RenderBlock<Cd> {
    fn render(&mut self, args: RenderArgs, draw_location: Point<f32>) {
        if let Some(rect) = &self.cached_background_rect {
            rect.draw(args.canvas, draw_location);
        } else {
            panic!("cannot render before layout")
        }

        for child in self.children.0.iter_mut() {
            child.element.borrow_mut().render_entry(args, draw_location);
        }
    }

    fn compute_layout(
        &mut self,
        LayoutInput {
            constraint,
            length_standard,
        }: LayoutInput,
    ) -> Size<f32> {
        let style = read_or_default(&self.style, &BlockStyle::DEFAULT);

        let resolve_rect = |rect: Rect<Length>| {
            rect.map_with(
                length_standard.as_ref().to_point().split_hv_to_rect(),
                |len, std| std.resolve(len).unwrap_or(0.0),
            )
        };

        let margin = resolve_rect(style.margin);
        let border = resolve_rect(style.border_width);
        let padding = resolve_rect(style.padding);

        let white_space_size = (margin + border + padding)
            .as_border_size()
            .map(|(start, end)| start + end)
            .to_size();

        let content_constraint =
            constraint.map_with(white_space_size, |mut constraint, white_space| {
                if let Some(num) = constraint.get_numerical_mut() {
                    *num = (*num - white_space).max(0.0);
                }
                constraint
            });

        let content_length_standard =
            length_standard.map_with(content_constraint, |ls, mut cons| {
                ls.set_percentage_reference(match cons.get_numerical_mut() {
                    Some(v) => *v,
                    None => 0.0,
                })
            });

        self.children
            .0
            .iter()
            .for_each(|e| e.element.borrow_mut().clear_layout_cache());

        let layouter = read_or_default(&self.layouter, &DefaultLayouter);
        let content_size = layouter.compute_layout(
            LayoutChildren::new(&mut self.children.0, &content_length_standard),
            content_constraint,
        );

        let outer_size = content_size + white_space_size;
        self.cached_background_rect = Some(DrawRRect::new(DrawRRectProps {
            background: style.background,
            border_color: style.border_color,
            border_radius: style.border_radius,
            margin,
            border,
            outer_size,
        }));

        outer_size
    }

    fn children_emit_event(&mut self, args: &mut EmitEventArgs) {
        for child in &self.children.0 {
            child.element.borrow_mut().emit_event(args);
        }
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn common(&self) -> &Common {
        &self.common
    }
}

pub struct ElementList<Cd>(Vec<Child<Cd>>);

impl<Cd> ElementList<Cd> {
    pub fn new() -> Self {
        ElementList(Vec::new())
    }

    pub fn push(&mut self, el: Element, child_data: Cd) {
        self.0.push(Child {
            element: el,
            child_data,
        });
    }
}
