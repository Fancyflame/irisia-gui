use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::{
    application::{content::GlobalContent, event_comp::NodeEventMgr},
    element::{props::SetStdStyles, Element, ElementUpdate, UpdateElement},
    event::EventDispatcher,
    structure::{slot::Slot, MapVisitor},
    style::StyleContainer,
    update_with::{SpecificUpdate, UpdateWith},
};

use super::{
    children::ChildrenNodes, data_structure::InsideRefCell, layer::SharedLayerCompositer,
    ElementModel, RcElementModel,
};

// add one

pub struct AddOne<El, Pr, Sty, Ch, Oc> {
    _el: PhantomData<El>,
    pub(crate) props: Pr,
    pub(crate) styles: Sty,
    pub(crate) children: Ch,
    pub(crate) on_create: Oc,
}

pub fn add_one<El, Pr, Sty, Ch, Oc>(
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

pub(crate) struct ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc> {
    pub(crate) add_one: AddOne<El, Pr, Sty, Ch, Oc>,
    pub(crate) content: EMUpdateContent<'a>,
}

#[derive(Clone)]
pub(crate) struct EMUpdateContent<'a> {
    pub(crate) global_content: &'a Arc<GlobalContent>,
    pub(crate) parent_layer: SharedLayerCompositer,
}

impl<'a, El, Pr, Sty, Ch, Oc> MapVisitor<AddOne<El, Pr, Sty, Ch, Oc>> for EMUpdateContent<'a> {
    type Output = ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc>;
    fn map_visit(&self, data: AddOne<El, Pr, Sty, Ch, Oc>) -> Self::Output {
        ElementModelUpdater {
            add_one: data,
            content: *self,
        }
    }
}

// impl update

type ImplUpdateElement<'a, El, Pr, Sty, Ch> =
    UpdateElement<'a, <Pr as SetStdStyles<'a, Sty>>::Output, El, Sty, <Ch as ChildrenNodes>::Model>;

impl<El, Pr, Sty, Ch, Oc> UpdateWith<ElementModelUpdater<'_, El, Pr, Sty, Ch, Oc>>
    for Rc<ElementModel<El, Sty, Ch::Model>>
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

        let props = props.set_std_styles(&styles);

        let this = Rc::new_cyclic(|weak| ElementModel {
            el: RwLock::new(None),
            global_content: global_content.clone(),
            ed: EventDispatcher::new(),
            in_cell: RefCell::new(InsideRefCell {
                styles,
                expanded_children: None,
                event_mgr: NodeEventMgr::new(),
                parent_layer: Rc::downgrade(&parent_layer),
                indep_layer: None,
            }),
            slot_cache: Slot::new(children.create_model(EMUpdateContent {
                global_content,
                parent_layer: parent_layer.clone(),
            })),
            draw_region: Default::default(),
            interact_region: Cell::new(None),
            acquire_independent_layer: Cell::new(false),
        });

        // hold the lock prevent from being accessed
        let mut write = this.el.blocking_write();
        *write = Some(El::el_create(&this, props));
        drop(write);

        on_create(&this);
        this
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

        let props = props.set_std_styles(&styles);

        let mut in_cell = self.in_cell.borrow_mut();
        in_cell.styles = styles;
        in_cell.parent_layer = Rc::downgrade(&parent_layer);

        children.update_model(
            &mut self.slot_cache.borrow_mut(),
            EMUpdateContent {
                global_content: &self.global_content,
                parent_layer: in_cell.get_children_layer(),
            },
            &mut equality_matters,
        );

        equality_matters
            & self
                .el_write_clean()
                .el_update(self, props, equality_matters)
    }
}

impl<'a, El, Pr, Sty, Ch, Oc> SpecificUpdate for ElementModelUpdater<'a, El, Pr, Sty, Ch, Oc>
where
    El: Element,
    Ch: ChildrenNodes,
{
    type UpdateTo = RcElementModel<El, Sty, Ch::Model>;
}
