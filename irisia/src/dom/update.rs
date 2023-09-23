use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::{Rc, Weak},
};

use tokio::sync::RwLock;

use crate::{
    application::{
        content::GlobalContent, event_comp::NodeEventMgr, redraw_scheduler::RedrawObject,
    },
    element::{props::SetStdStyles, Element, ElementUpdate},
    event::EventDispatcher,
    structure::{slot::Slot, MapVisitor},
    style::StyleContainer,
    update_with::{SpecificUpdate, UpdateWith},
};

use super::{
    children::ChildrenNodes, data_structure::InsideRefCell, layer::LayerCompositer, DropProtection,
    ElementModel,
};

// add one

pub struct AddOne<El, Pr, Sty, Ch, Oc> {
    _el: PhantomData<El>,
    pub(crate) props: Pr,
    pub(crate) styles: Sty,
    pub(crate) children: Ch,
    pub(crate) on_create: Oc,
}

pub fn one_child<El, Pr, Sty, Ch, Oc>(
    props: Pr,
    styles: Sty,
    children: Ch,
    on_create: Oc,
) -> AddOne<El, Pr, Sty, Ch, Oc> {
    AddOne {
        _el: Default::default(),
        props,
        styles,
        children,
        on_create,
    }
}

// element model update content

pub struct ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc> {
    pub(crate) add_one: AddOne<El, Pr, Sty, Ch, Oc>,
    pub(crate) content: EMUpdateContent<'a>,
}

#[derive(Clone)]
pub struct EMUpdateContent<'a> {
    pub(crate) global_content: &'a Rc<GlobalContent>,
    pub(crate) parent_layer: Option<Weak<dyn RedrawObject>>,
}

impl<'a, El, Pr, Sty, Ch, Oc> MapVisitor<AddOne<El, Pr, Sty, Ch, Oc>> for EMUpdateContent<'a> {
    type Output = ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc>;
    fn map_visit(&self, data: AddOne<El, Pr, Sty, Ch, Oc>) -> Self::Output {
        ElementModelUpdater {
            add_one: data,
            content: self.clone(),
        }
    }
}

// impl update

impl<El, Pr, Sty, Ch, Oc> UpdateWith<ElementModelUpdater<'_, El, Pr, Sty, Ch, Oc>>
    for DropProtection<El, Sty, Ch::Model>
where
    Pr: for<'sty> SetStdStyles<'sty, Sty>,
    El: Element + for<'sty> ElementUpdate<<Pr as SetStdStyles<'sty, Sty>>::Output>,
    Sty: StyleContainer + 'static,
    Ch: ChildrenNodes,
    Oc: FnOnce(&Rc<ElementModel<El, Sty, Ch::Model>>),
{
    fn create_with(updater: ElementModelUpdater<El, Pr, Sty, Ch, Oc>) -> Self {
        let ElementModelUpdater {
            add_one:
                AddOne {
                    _el: _,
                    props,
                    styles,
                    children,
                    on_create,
                },
            content:
                EMUpdateContent {
                    global_content,
                    parent_layer,
                },
        } = updater;

        let this = Rc::new_cyclic(|weak: &Weak<ElementModel<_, _, _>>| ElementModel {
            this: weak.clone(),
            el: RwLock::new(None),
            el_alive: Cell::new(true),
            global_content: global_content.clone(),
            ed: EventDispatcher::new(),
            in_cell: RefCell::new(InsideRefCell {
                styles,
                expanded_children: None,
                event_mgr: NodeEventMgr::new(),
                parent_layer: parent_layer.clone(),
                indep_layer: if parent_layer.is_some() {
                    None
                } else {
                    Some(LayerCompositer::new())
                },
            }),
            slot_cache: Slot::new(children.create_model(EMUpdateContent {
                global_content,
                parent_layer: Some(parent_layer.unwrap_or_else(|| weak.clone())),
            })),
            draw_region: Default::default(),
            interact_region: Cell::new(None),
            acquire_independent_layer: Cell::new(false),
        });

        // hold the lock prevent from being accessed
        let mut write = this.el.blocking_write();
        let el = El::el_create(&this, props.set_std_styles(&this.in_cell.borrow().styles));
        el.set_children(&this);
        *write = Some(el);
        drop(write);

        on_create(&this);
        this.set_dirty();
        DropProtection(this)
    }

    fn update_with(
        &mut self,
        updater: ElementModelUpdater<El, Pr, Sty, Ch, Oc>,
        mut equality_matters: bool,
    ) -> bool {
        let ElementModelUpdater {
            add_one:
                AddOne {
                    _el: _,
                    props,
                    styles,
                    children,
                    on_create: _,
                },
            content:
                EMUpdateContent {
                    global_content: _,
                    parent_layer,
                },
        } = updater;

        let mut in_cell = self.in_cell.borrow_mut();
        in_cell.styles = styles;
        in_cell.parent_layer = parent_layer;

        children.update_model(
            &mut self.slot_cache.borrow_mut(),
            EMUpdateContent {
                global_content: &self.global_content,
                parent_layer: Some(self.get_children_layer(&mut in_cell)),
            },
            &mut equality_matters,
        );

        let mut el = self.el_write_clean();
        let unchanged = equality_matters
            & el.el_update(
                &self.this.upgrade().unwrap(),
                props.set_std_styles(&in_cell.styles),
                equality_matters,
            );
        el.set_children(self);
        unchanged
    }
}

impl<'a, El, Pr, Sty, Ch, Oc> SpecificUpdate for ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Ch: ChildrenNodes,
{
    type UpdateTo = DropProtection<El, Sty, Ch::Model>;
}
