use tokio::sync::RwLock;

use crate::{
    application::event_comp::NodeEventMgr, element::ElementCreate, event::EventDispatcher,
    style::style_box::IntoStyleBox, Element,
};

use super::{
    data_structure::{Context, InsideRefCell},
    ElementModel, RcElementModel,
};

use std::{
    cell::{Cell, RefCell},
    ops::Deref,
    rc::{Rc, Weak},
};

impl<El> ElementModel<El>
where
    El: Element,
{
    pub fn new<Pr, Isb, _Sln, Oc>(
        props: Pr,
        styles: Isb,
        slot: El::Slot,
        on_create: Oc,
    ) -> DropProtection<El>
    where
        Isb: IntoStyleBox<_Sln>,
        El: ElementCreate<Pr>,
        Oc: FnOnce(&RcElementModel<El>),
    {
        let ed = EventDispatcher::new();

        let this = Rc::new_cyclic(|weak: &Weak<ElementModel<El>>| ElementModel {
            this: weak.clone(),
            el: RwLock::new(None),
            ed: ed.clone(),
            in_cell: RefCell::new(InsideRefCell {
                styles: styles.into_box(),
                event_mgr: NodeEventMgr::new(ed),
                indep_layer: None,
                context: Context::None,
            }),
            draw_region: Default::default(),
            interact_region: Cell::new(None),
            acquire_independent_layer: Cell::new(false),
        });

        // hold the lock prevent from being accessed
        let mut write = this.el.try_write().unwrap();
        let el = El::el_create(&this, props, slot);
        *write = Some(el);
        drop(write);

        on_create(&this);
        this.set_dirty();
        DropProtection(this)
    }
}

pub struct DropProtection<El>(pub(crate) RcElementModel<El>)
where
    El: Element;

impl<El> Deref for DropProtection<El>
where
    El: Element,
{
    type Target = RcElementModel<El>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<El> Drop for DropProtection<El>
where
    El: Element,
{
    fn drop(&mut self) {
        self.0.set_abandoned();
    }
}
