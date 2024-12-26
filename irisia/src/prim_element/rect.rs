use std::{cell::RefCell, rc::Rc};

use irisia_backend::skia_safe::{Color, Color4f, Paint, RRect, Rect as SkRect};

use crate::{el_model::EMCreateCtx, model2::VModel, primitive::Region};

use super::{Element, GetElement, Handle, RenderTree};

pub struct RenderRect {
    rect: Rect,
    // ctx: EMCreateCtx,
    prev_draw_region: Option<Region>,
}

impl RenderTree for RenderRect {
    fn render(&mut self, args: super::RenderArgs, draw_region: Region) {
        if !args.needs_redraw(draw_region) {
            return;
        }

        self.prev_draw_region = Some(draw_region);
        let border_radius = self.rect.border_radius;
        let rrect = RRect::new_nine_patch(
            SkRect {
                left: draw_region.left_top.0,
                top: draw_region.left_top.1,
                right: draw_region.right_bottom.0,
                bottom: draw_region.right_bottom.1,
            },
            border_radius[0],
            border_radius[1],
            border_radius[2],
            border_radius[3],
        );
        let paint = Paint::new(Color4f::from(self.rect.color), None);

        args.canvas.draw_rrect(rrect, &paint);
    }
}

#[derive(Default, PartialEq, Clone)]
pub struct Rect {
    pub color: Color,
    pub border_radius: [f32; 4],
}

pub struct RectModel {
    node: Handle<RenderRect>,
}

impl VModel for Rect {
    type Storage = RectModel;
    fn create(&self, _: &crate::el_model::EMCreateCtx) -> Self::Storage {
        let render = RenderRect {
            rect: self.clone(),
            // ctx: ctx.clone(),
            prev_draw_region: None,
        };

        RectModel {
            node: Rc::new(RefCell::new(render)),
        }
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &crate::el_model::EMCreateCtx) {
        let mut node = storage.node.borrow_mut();

        if node.rect == *self {
            return;
        }

        node.rect = self.clone();
        if let Some(dr) = node.prev_draw_region.take() {
            ctx.global_content.request_redraw(dr);
        }
    }
}

impl GetElement for RectModel {
    fn get_element(&self) -> super::Element {
        Element::Rect(self.node.clone())
    }
}
