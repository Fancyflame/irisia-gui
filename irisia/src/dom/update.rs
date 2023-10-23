use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
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
    layer::LayerCompositer,
    ChildNodes, DropProtection, ElementModel,
};

// impl update

impl<El, Sty, Sc> ElementModel<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: ChildNodes,
{
    pub fn new<Pr, Oc>(
        props: Pr,
        styles: Sty,
        slot: Sc,
        on_create: Oc,
    ) -> DropProtection<El, Sty, Sc>
    where
        Pr: for<'a> SetStdStyles<'a, Sty>,
        El: ElementCreate<Pr>,
        Oc: FnOnce(&Rc<ElementModel<El, Sty, Sc>>),
    {
        let ed = EventDispatcher::new();

        let this = Rc::new_cyclic(|weak: &Weak<ElementModel<_, _, _>>| ElementModel {
            this: weak.clone(),
            el: RwLock::new(None),
            ed: ed.clone(),
            in_cell: RefCell::new(InsideRefCell {
                styles,
                expanded_children: None,
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
        let el = El::el_create(&this, props.set_std_styles(&this.in_cell.borrow().styles));
        el.set_children(&this);
        *write = Some(el);
        drop(write);

        on_create(&this);
        this.set_dirty();
        DropProtection(this)
    }
}
