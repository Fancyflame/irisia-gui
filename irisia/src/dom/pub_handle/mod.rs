use irisia_backend::WinitWindow;
use std::{cell::RefMut, future::Future, rc::Rc};
use tokio::{
    sync::{RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard},
    task::JoinHandle,
};

use crate::{
    application::content::GlobalContent,
    event::{standard::ElementAbandoned, EdProvider, EventDispatcher, Listen},
    primitive::Region,
    style::StyleContainer,
    Element, StyleReader,
};

pub use self::{layout_el::LayoutElements, write_guard::ElWriteGuard};

use super::{
    children::{ChildrenBox, ChildrenNodes},
    data_structure::ElementModel,
    EMUpdateContent, RcElementModel, RenderMultiple,
};

mod layout_el;
mod write_guard;

impl<El, Sty, Sc> ElementModel<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: RenderMultiple + 'static,
{
    /// Get a write guard without setting dirty
    pub(super) fn el_write_clean(&self) -> RwLockMappedWriteGuard<El> {
        RwLockWriteGuard::map(
            self.el.try_write().expect(
                "do not hold a element write guard across `await`. \
                this limitation will be lifted in the future, but still \
                discouraged",
            ),
            |x| x.as_mut().unwrap(),
        )
    }

    /// Get a write guard of this element and setting dirty.
    /// `None` if this element will no longer used.
    pub async fn el_write<'a>(self: &'a Rc<Self>) -> Option<ElWriteGuard<'a, El, Rc<Self>>> {
        RwLockWriteGuard::try_map(self.el.write().await, |x| x.as_mut())
            .ok()
            .map(|guard| ElWriteGuard {
                write: guard,
                set_dirty: self,
            })
    }

    /// Get a read guard of this element and dirty flag is not affected.
    /// `None` if this element will no longer used.
    pub async fn el_read(&self) -> Option<RwLockReadGuard<El>> {
        RwLockReadGuard::try_map(self.el.read().await, |x| x.as_ref()).ok()
    }

    pub fn alive(&self) -> bool {
        self.el_alive.get()
    }

    /// Listen event with options
    pub fn listen<'a>(self: &'a Rc<Self>) -> Listen<'a, Rc<Self>, (), (), (), ()> {
        Listen::new(self)
    }

    pub fn slot(&self) -> impl ChildrenNodes + '_ {
        &self.slot_cache
    }

    /// Get event dispatcher of this element.
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.ed
    }

    /// Let this element being focused on.
    pub fn focus(&self) {
        self.global_content.focusing().focus(self.ed.clone());
    }

    /// Let this element no longer being focused. does nothing if
    /// this element is not in focus.
    pub fn blur(&self) {
        self.global_content.focusing().blur_checked(&self.ed);
    }

    /// Get global content of the window.
    pub fn global(&self) -> &Rc<GlobalContent> {
        &self.global_content
    }

    /// Get the raw window. Alias to `self.global().window()`.
    pub fn window(&self) -> &WinitWindow {
        self.global_content.window()
    }

    /// Get the region for drawing. Specified by parent element,
    /// and can only be changed by parent element.
    pub fn draw_region(&self) -> Region {
        self.draw_region.get()
    }

    /// Get styles bind to this element
    pub fn styles<Sr>(&self) -> Sr
    where
        Sty: StyleContainer,
        Sr: StyleReader,
    {
        self.in_cell.borrow().styles.read()
    }

    /// Set dirty flag to `true`.
    pub fn set_dirty(&self) {
        self.global_content.request_redraw(
            self.get_children_layer(&self.in_cell.borrow())
                .upgrade()
                .expect("parent layer unexpectedly dropped"),
        )
    }

    /// Query whether independent layer was acquired.
    ///
    /// Note that this function will NOT reflect the actual state of
    /// the layer. Independent layer may not exists while result is `true`
    /// and may exists while result is `false`.
    pub fn indep_layer_acquired(&self) -> bool {
        self.acquire_independent_layer.take()
    }

    /// Set `true` to acquire independent render layer for performance optimizations.
    /// Duplicated acquirement will be ignored.
    ///
    /// A independent render layer will cause a heap allocation to render this
    /// element and its children elements rather than drawing on parent's render
    /// layer.
    ///
    /// Recommended when playing animation.
    pub fn acquire_indep_layer(&self, acquire: bool) {
        if self.indep_layer_acquired() == acquire {
            return;
        }
        self.acquire_independent_layer.set(acquire);
        self.set_dirty();
    }

    /// Spwan a daemon task on `fut`.
    ///
    /// The spawned task will be cancelled when element dropped,
    /// or can be cancelled manually.
    pub fn daemon<F>(&self, fut: F) -> JoinHandle<()>
    where
        F: Future + 'static,
    {
        let ed = self.ed.clone();
        tokio::task::spawn_local(async move {
            tokio::select! {
                _ = ed.recv_trusted::<ElementAbandoned>() => {},
                _ = fut => {}
            }
        })
    }

    pub fn layout_children(&self) -> Option<LayoutElements> {
        self.set_dirty();
        RefMut::filter_map(self.in_cell.borrow_mut(), |in_cell| {
            in_cell
                .expanded_children
                .as_mut()
                .map(|x| x.as_render_multiple())
        })
        .ok()
        .map(|refmut| LayoutElements { refmut })
    }

    pub fn set_children<'a, Ch>(self: &'a Rc<Self>, children: Ch) -> LayoutElements<'a>
    where
        Ch: ChildrenNodes,
    {
        self.set_dirty();
        let in_cell = self.in_cell.borrow_mut();

        let updater = EMUpdateContent {
            global_content: &self.global_content,
            parent_layer: Some(self.get_children_layer(&in_cell)),
        };

        let children_box = RefMut::map(in_cell, |x| &mut x.expanded_children);

        let refmut = RefMut::map(children_box, |option| match option {
            Some(cb) => {
                let model=
                    cb.as_render_multiple()
                    .as_any()
                    .downcast_mut::<Ch::Model>()
                    .expect("the type of children is not equal to previous's, these two is expected to be the same");

                children.update_model(model, updater, &mut false);
                model
            }
            place @ None => place
                .insert(ChildrenBox::new(children.create_model(updater)))
                .as_render_multiple()
                .as_any()
                .downcast_mut()
                .unwrap(),
        });

        LayoutElements { refmut }
    }
}

impl<El, Sty, Sc> EdProvider for RcElementModel<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: RenderMultiple + 'static,
{
    fn event_dispatcher(&self) -> &EventDispatcher {
        ElementModel::event_dispatcher(self)
    }

    fn daemon<F>(&self, f: F) -> JoinHandle<()>
    where
        F: Future + 'static,
    {
        ElementModel::daemon(self, f)
    }

    fn handle_available(&self) -> bool {
        self.alive()
    }
}
