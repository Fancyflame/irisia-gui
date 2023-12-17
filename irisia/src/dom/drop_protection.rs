use tokio::sync::RwLock;

use crate::{
    application::event_comp::NodeEventMgr, element::ElementCreate, event::EventDispatcher,
    structure::Slot, style::StyleGroup, Element,
};

use super::{
    data_structure::{Context, InsideRefCell},
    ChildNodes, ElementModel, RcElementModel,
};

use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    rc::{Rc, Weak},
};

impl<El, Sty, Slt> ElementModel<El, Sty, Slt>
where
    El: Element,
    Sty: StyleGroup + 'static,
    Slt: ChildNodes,
{
    pub fn new<Pr, Oc>(
        props: Pr,
        styles: Sty,
        slot: Slt,
        on_create: Oc,
    ) -> DropProtection<El, Sty, Slt>
    where
        El: ElementCreate<Pr>,
        Oc: FnOnce(&RcElementModel<El, Sty, Slt>),
    {
        let ed = EventDispatcher::new();
        let slot = Slot::new(slot);
        let (el, cb) = El::el_create(props, slot.private_clone());

        let this = Rc::new_cyclic(|weak: &Weak<_>| ElementModel {
            this: weak.clone(),
            standalone_render: weak.clone() as _,
            el: RwLock::new(Some(el)),
            ed: ed.clone(),
            in_cell: RefCell::new(InsideRefCell {
                children: cb,
                styles,
                event_mgr: NodeEventMgr::new(ed),
                indep_layer: None,
                context: Context::None,
            }),
            draw_region: Default::default(),
            interact_region: Cell::new(None),
            flag_dirty_setted: Cell::new(false),
            acquire_independent_layer: Cell::new(false),
        });

        El::on_created(&this);
        on_create(&this);
        this.set_dirty();
        DropProtection(this)
    }
}

pub struct DropProtection<El, Sty, Slt>(pub(crate) RcElementModel<El, Sty, Slt>)
where
    RcElementModel<El, Sty, Slt>: 'static;

impl<El, Sty, Slt> Deref for DropProtection<El, Sty, Slt> {
    type Target = RcElementModel<El, Sty, Slt>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<El, Sty, Slt> Drop for DropProtection<El, Sty, Slt> {
    fn drop(&mut self) {
        self.0.set_abandoned();
    }
}
