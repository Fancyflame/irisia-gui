use std::{marker::PhantomData, rc::Rc, time::Duration};

use anyhow::anyhow;

use crate::{
    application::{event_comp::IncomingPointerEvent, redraw_scheduler::StandaloneRender},
    dom::{data_structure::Context, layer::LayerRebuilder, DropProtection},
    element::{Element, GlobalContent},
    primitive::Region,
    structure::{VisitBy, VisitOn},
    Result, StyleReader,
};

pub use self::render_element::RenderElement;
use super::data_structure::AttachedCtx;

mod render_element;

pub trait ChildNodes: VisitBy + 'static {
    fn render<'a, 'lr>(&self, re: &'a mut RenderElement<'_, 'lr>) -> Result<()> {
        self.visit_by(&mut RenderHelper {
            lr: re.lr,
            interval: re.interval,
        })
    }

    fn peek_styles<F, Sr>(&self, f: F)
    where
        F: FnMut(Sr),
        Sr: StyleReader,
    {
        let _ = self.visit_by(&mut PeekStyles {
            map: f,
            _sr: PhantomData,
        });
    }

    fn len(&self) -> usize {
        VisitBy::len(self)
    }

    fn layout<F, Sr>(&self, f: F) -> Result<()>
    where
        F: FnMut(Sr) -> Option<Region>,
        Sr: StyleReader,
    {
        self.visit_by(&mut LayoutHelper {
            map: f,
            _sr: PhantomData,
        })
    }

    fn emit_event(&self, ipe: &IncomingPointerEvent) -> bool {
        let mut eeh = EmitEventHelper {
            children_entered: false,
            ipe,
        };
        let _ = self.visit_by(&mut eeh);
        eeh.children_entered
    }
}

struct RenderHelper<'a, 'lr> {
    lr: &'a mut LayerRebuilder<'lr>,
    interval: Duration,
}

impl VisitOn for RenderHelper<'_, '_> {
    fn visit_on<El>(&mut self, data: &DropProtection<El>) -> Result<()>
    where
        El: Element,
    {
        data.build_layers(self.lr, self.interval)
    }
}

struct LayoutHelper<F, Sr> {
    map: F,
    _sr: PhantomData<Sr>,
}

impl<F, Sr> VisitOn for LayoutHelper<F, Sr>
where
    F: FnMut(Sr) -> Option<Region>,
    Sr: StyleReader,
{
    fn visit_on<El>(&mut self, data: &DropProtection<El>) -> Result<()>
    where
        El: Element,
    {
        let region = (self.map)(data.in_cell.borrow().styles.read());
        match region {
            Some(region) => {
                data.set_draw_region(region);
                Ok(())
            }
            None => Err(anyhow!("unexpected end of layouter")),
        }
    }
}

struct PeekStyles<F, Sr> {
    map: F,
    _sr: PhantomData<Sr>,
}

impl<F, Sr> VisitOn for PeekStyles<F, Sr>
where
    F: FnMut(Sr),
    Sr: StyleReader,
{
    fn visit_on<El>(&mut self, data: &DropProtection<El>) -> Result<()>
    where
        El: Element,
    {
        (self.map)(data.in_cell.borrow().styles.read());
        Ok(())
    }
}

struct EmitEventHelper<'a, 'root> {
    ipe: &'a IncomingPointerEvent<'root>,
    children_entered: bool,
}

impl VisitOn for EmitEventHelper<'_, '_> {
    fn visit_on<El>(&mut self, data: &DropProtection<El>) -> Result<()>
    where
        El: Element,
    {
        self.children_entered |= data.emit_event(self.ipe);
        Ok(())
    }
}

struct AttachHelper<'a> {
    parent_layer: Option<&'a Rc<dyn StandaloneRender>>,
    global_content: Rc<GlobalContent>,
}

impl VisitOn for AttachHelper<'_> {
    fn visit_on<El>(&mut self, data: &DropProtection<El>) -> Result<()>
    where
        El: Element,
    {
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
