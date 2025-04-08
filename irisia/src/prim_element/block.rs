use crate::{
    application::event2::pointer_event::PointerStateDelta,
    primitive::{Point, Region},
};

use super::{
    redraw_guard::RedrawGuard, Common, EMCreateCtx, Element, EventCallback, GetElement, Handle,
    RenderTree,
};

pub type LayoutFn = fn(Point, &[Element], &mut Vec<Region>);

pub struct RenderBlock {
    tree: Tree,
    children_draw_regions: Vec<Region>,
    common: Common,
}

#[derive(Clone)]
pub struct Tree {
    pub layout_fn: LayoutFn,
    pub children: Vec<Element>,
}

impl RenderBlock {
    pub fn new(tree: Tree, event_callback: EventCallback, ctx: &EMCreateCtx) -> Self {
        Self {
            tree,
            children_draw_regions: vec![],
            common: Common::new(event_callback, ctx),
        }
    }

    pub fn update_tree(&mut self) -> RedrawGuard<Tree> {
        RedrawGuard::new(&mut self.tree, &mut self.common)
    }
}

impl RenderTree for RenderBlock {
    fn render(&mut self, args: super::RenderArgs, draw_region: Region) {
        if !args.needs_redraw(draw_region) {
            return;
        }

        let area = draw_region.right_bottom - draw_region.left_top;
        if self.common.prev_draw_region != Some(draw_region) {
            self.children_draw_regions.clear();
            (self.tree.layout_fn)(area, &self.tree.children, &mut self.children_draw_regions);
            assert_eq!(
                self.tree.children.len(),
                self.children_draw_regions.len(),
                "given layouted count mismatch"
            );
            self.common.prev_draw_region = Some(draw_region);
        }

        for (el, relative_region) in self
            .tree
            .children
            .iter_mut()
            .zip(self.children_draw_regions.iter())
        {
            let absolute_region = Region {
                left_top: relative_region.left_top + draw_region.left_top,
                right_bottom: relative_region.right_bottom + draw_region.left_top,
            };
            el.render(args, absolute_region);
        }
    }

    fn emit_event(&mut self, delta: &mut PointerStateDelta, draw_region: Region) {
        for (el, &child_draw_region) in self
            .tree
            .children
            .iter_mut()
            .zip(self.children_draw_regions.iter())
            .rev()
        {
            el.emit_event(delta, child_draw_region);
        }

        self.common.use_callback(delta, draw_region);
    }

    fn set_callback(&mut self, callback: EventCallback) {
        self.common.event_callback = callback;
    }
}

impl GetElement for Handle<RenderBlock> {
    fn get_element(&self) -> Element {
        Element::Block(self.clone())
    }
}
