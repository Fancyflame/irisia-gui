use std::{rc::Rc, time::Duration};

use irisia_backend::skia_safe::Canvas;

use crate::{application::redraw_scheduler::RedrawObject, element::RenderElement, Element, Result};

use super::{children::ChildrenBox, ElementModel};

fn unwrap_children(cb: &mut Option<ChildrenBox>) -> &mut ChildrenBox {
    cb.as_mut()
        .unwrap_or_else(|| unreachable!("children not initialized"))
}

impl<El, Sty, Sc> RedrawObject for Rc<ElementModel<El, Sty, Sc>>
where
    El: Element,
{
    fn redraw(&self, canvas: &mut Canvas, interval: Duration) -> Result<()> {
        let mut in_cell = self.in_cell.borrow_mut();
        let mut lr = in_cell
            .indep_layer
            .as_mut()
            .expect("independent layer required")
            .borrow_mut()
            .rebuild(canvas);

        self.el_write_clean().render(RenderElement::new(
            &mut lr,
            in_cell
                .expanded_children
                .as_mut()
                .unwrap()
                .as_render_multiple(),
            interval,
        ))
    }
}
