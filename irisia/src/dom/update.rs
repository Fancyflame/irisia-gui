use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use tokio::sync::RwLock;

use crate::{
    application::event_comp::NodeEventMgr,
    element::{props::SetStdStyles, Element, ElementCreate},
    event::EventDispatcher,
    style::StyleContainer,
};

use super::{
    data_structure::{Context, InsideRefCell},
    DropProtection, ElementModel,
};

// impl update

impl<El, Sty> ElementModel<El, Sty>
where
    El: Element,
    Sty: StyleContainer + 'static,
{
    pub fn new<Pr, Oc>(
        props: Pr,
        styles: Sty,
        slot: El::Slot,
        on_create: Oc,
    ) -> DropProtection<El, Sty>
    where
        Pr: for<'a> SetStdStyles<&'a Sty>,
        El: for<'a> ElementCreate<<Pr as SetStdStyles<&'a Sty>>::Output>,
        Oc: FnOnce(&Rc<ElementModel<El, Sty>>),
    {
        let ed = EventDispatcher::new();

        let this = Rc::new_cyclic(|weak: &Weak<ElementModel<_, _>>| ElementModel {
            this: weak.clone(),
            el: RwLock::new(None),
            ed: ed.clone(),
            in_cell: RefCell::new(InsideRefCell {
                styles,
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
        let el = El::el_create(
            &this,
            props.set_std_styles(&this.in_cell.borrow().styles),
            slot,
        );
        *write = Some(el);
        drop(write);

        on_create(&this);
        this.set_dirty();
        DropProtection(this)
    }
}
