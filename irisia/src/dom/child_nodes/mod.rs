use std::{rc::Rc, time::Duration};

use anyhow::anyhow;

use crate::{
    application::{event_comp::IncomingPointerEvent, redraw_scheduler::StandaloneRender},
    dom::{data_structure::Context, layer::LayerRebuilder},
    element::GlobalContent,
    primitive::Region,
    structure::{VisitBy, VisitOn},
    style::style_box::RawStyleGroup,
    ElModel, Result, StyleReader,
};

pub use self::render_element::RenderElement;
use super::data_structure::AttachedCtx;

mod render_element;

type TypeElimatedSrGroup<'a> = &'a mut dyn FnMut(&dyn RawStyleGroup);
type TypeElimatedLayouter<'a> = &'a mut dyn FnMut(&dyn RawStyleGroup, usize) -> Option<Region>;

pub trait ChildNodes: RawChildNodes {
    fn render<'a, 'lr>(&self, re: &'a mut RenderElement<'_, 'lr>) -> Result<()> {
        self.render_raw(re)
    }

    fn peek_styles<F, Sr>(&self, mut f: F)
    where
        F: FnMut(Sr),
        Sr: StyleReader,
    {
        self.peek_styles_raw(&mut |rsg| f(Sr::read_style(&rsg)))
    }

    fn len(&self) -> usize {
        self.len_raw()
    }

    fn layout<F, Sr>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(Sr, usize) -> Option<Region>,
        Sr: StyleReader,
    {
        self.layout_raw(&mut |rsg, nth| f(Sr::read_style(rsg), nth))
    }

    fn emit_event(&self, ipe: &IncomingPointerEvent) -> bool {
        self.emit_event_raw(ipe)
    }
}

pub trait RawChildNodes: 'static {
    fn render_raw<'a, 'lr>(&self, re: &'a mut RenderElement<'_, 'lr>) -> Result<()>;
    fn peek_styles_raw(&self, f: TypeElimatedSrGroup);
    fn len_raw(&self) -> usize;
    fn layout_raw(&self, iter: TypeElimatedLayouter) -> Result<()>;
    fn emit_event_raw(&self, ipe: &IncomingPointerEvent) -> bool;
}

impl<T: VisitBy + 'static> RawChildNodes for T {
    fn render_raw<'a, 'lr>(&self, re: &'a mut RenderElement<'_, 'lr>) -> Result<()> {
        self.visit_by(&mut RenderHelper {
            lr: re.lr,
            interval: re.interval,
        })
    }

    fn peek_styles_raw(&self, f: TypeElimatedSrGroup) {
        self.visit_by(&mut PeekStyles { reader: f }).unwrap();
    }

    fn len_raw(&self) -> usize {
        VisitBy::len(self)
    }

    fn layout_raw(&self, layouter: TypeElimatedLayouter) -> Result<()> {
        self.visit_by(&mut LayoutHelper { nth: 0, layouter })
    }

    fn emit_event_raw(&self, ipe: &IncomingPointerEvent) -> bool {
        let mut eeh = EmitEventHelper {
            children_entered: false,
            ipe,
        };
        let _ = self.visit_by(&mut eeh);
        eeh.children_entered
    }
}

impl<T: RawChildNodes> ChildNodes for T {}

struct RenderHelper<'a, 'lr> {
    lr: &'a mut LayerRebuilder<'lr>,
    interval: Duration,
}

impl VisitOn for RenderHelper<'_, '_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        data.build_layers(self.lr, self.interval)
    }
}

struct LayoutHelper<'a> {
    nth: usize,
    layouter: TypeElimatedLayouter<'a>,
}

impl VisitOn for LayoutHelper<'_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        let region = (self.layouter)(&data.in_cell.borrow().styles, self.nth);
        match region {
            Some(region) => {
                self.nth += 1;
                data.set_draw_region(region);
                Ok(())
            }
            None => Err(anyhow!("unexpected end of layouter")),
        }
    }
}

struct PeekStyles<'a> {
    reader: TypeElimatedSrGroup<'a>,
}

impl VisitOn for PeekStyles<'_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        (self.reader)(&data.in_cell.borrow().styles);
        Ok(())
    }
}

struct EmitEventHelper<'a, 'root> {
    ipe: &'a IncomingPointerEvent<'root>,
    children_entered: bool,
}

impl VisitOn for EmitEventHelper<'_, '_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        self.children_entered |= data.emit_event(self.ipe);
        Ok(())
    }
}

struct AttachHelper<'a> {
    parent_layer: Option<&'a Rc<dyn StandaloneRender>>,
    global_content: Rc<GlobalContent>,
}

impl VisitOn for AttachHelper<'_> {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()> {
        match &mut data.in_cell.borrow_mut().context {
            ctx @ Context::None => {
                *ctx = Context::Attached(AttachedCtx {
                    global_content: self.global_content.clone(),
                    parent_layer: self.parent_layer.map(Rc::downgrade),
                });
                Ok(())
            }

            Context::Attached(AttachedCtx {
                global_content,
                parent_layer,
            }) => {
                if !Rc::ptr_eq(global_content, &self.global_content) {
                    return Err(anyhow!(
                        "cannot attach element node in another context to this"
                    ));
                }
                *parent_layer = self.parent_layer.map(Rc::downgrade);
                Ok(())
            }

            Context::Destroyed => Err(anyhow!("cannot update context to an abondoned element")),
        }
    }
}
