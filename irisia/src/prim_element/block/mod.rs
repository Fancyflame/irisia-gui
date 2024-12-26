use crate::primitive::{Point, Region};

use super::{Element, RenderTree};

pub mod vmodel;

pub type LayoutFn = fn(Point, &Vec<Element>, &mut Vec<Region>);

pub struct RenderBlock {
    layout_fn: LayoutFn,
    need_relayout: bool,
    prev_draw_region: Option<Region>,
    children: Vec<Element>,
    children_draw_regions: Vec<Region>,
}

impl RenderTree for RenderBlock {
    fn render(&mut self, args: super::RenderArgs, draw_region: Region) {
        if !args.needs_redraw(draw_region) {
            return;
        }

        let area = draw_region.right_bottom - draw_region.left_top;
        if self.prev_draw_region != Some(draw_region) || self.need_relayout {
            self.children_draw_regions.clear();
            (self.layout_fn)(area, &self.children, &mut self.children_draw_regions);
            assert_eq!(
                self.children.len(),
                self.children_draw_regions.len(),
                "given layouted count mismatch"
            );
            self.prev_draw_region = Some(draw_region);
            self.need_relayout = false;
        }

        for (el, relative_region) in self
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
}
