use crate::primitive::{Point, Region};

use super::{
    redraw_guard::RedrawGuard, Common, EMCreateCtx, Element, EmitEventArgs, EventCallback,
    GetElement, Handle, RenderTree,
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
    pub fn new(tree: Tree, event_callback: Option<EventCallback>, ctx: &EMCreateCtx) -> Self {
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

        // TODO: 如果渲染矩形大小相同，可以不用重新布局，直接平移全部矩形
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

    fn emit_event(&mut self, mut args: EmitEventArgs) {
        for (el, &child_draw_region) in self
            .tree
            .children
            .iter_mut()
            .zip(self.children_draw_regions.iter())
            .rev()
        {
            el.emit_event(args.reborrow(child_draw_region));
        }

        self.common.use_callback(args);
    }

    fn set_callback(&mut self, callback: EventCallback) {
        self.common.event_callback = Some(callback);
    }
}

impl GetElement for Handle<RenderBlock> {
    fn get_element(&self) -> Element {
        Element::Block(self.clone())
    }
}
