use std::future::Future;
use std::sync::{Arc, Weak};

use crate::event::{Event, EventDispatcher, EventReceive};

use irisia_backend::window_handle::close_handle::CloseHandle;
use irisia_backend::WinitWindow;
use tokio::sync::Mutex;

use super::ElementHandle;

pub mod element_handle;

pub struct InitContent<T: ?Sized> {
    pub(crate) _prevent_user_init: (),
    pub app: Weak<Mutex<T>>,
    pub element_handle: ElementHandle,
    pub window_event_dispatcher: EventDispatcher,
    pub window: Arc<WinitWindow>,
    pub close_handle: CloseHandle,
}

impl<T: ?Sized> Clone for InitContent<T> {
    fn clone(&self) -> Self {
        Self {
            _prevent_user_init: (),
            app: self.app.clone(),
            element_handle: self.element_handle.clone(),
            window_event_dispatcher: self.window_event_dispatcher.clone(),
            window: self.window.clone(),
            close_handle: self.close_handle,
        }
    }
}

impl<T: ?Sized> InitContent<T> {
    pub fn recv<E: Event>(&self) -> EventReceive<E> {
        self.element_handle.recv()
    }

    pub async fn recv_sys<E: Event>(&self) -> E {
        self.element_handle.recv_sys().await
    }

    pub async fn once<'a, Fut, Cb, CbFut>(&'a self, f: Fut, callback: Cb)
    where
        Fut: Future + Send,
        Fut::Output: Send,
        Cb: (FnOnce(&'a Self, Fut::Output) -> CbFut) + Send,
        CbFut: Future,
        T: Send,
    {
        let future = f.await;
        callback(self, future).await;
    }

    pub async fn once3<'a, Fut, Cb, CbFut>(&'a self, f: Fut, callback: Cb)
    where
        Fut: Future + Send,
        Fut::Output: Send,
        Cb: (FnOnce(&'a Self, &mut T, Fut::Output) -> CbFut) + Send,
        CbFut: Future,
        T: Send,
    {
        let Some(app) = self.app.upgrade()
        else {
            return;
        };

        let future = f.await;
        let mut guard = app.lock().await;
        callback(self, &mut guard, future).await;
    }

    pub async fn on<'a, F, Fut, Cb, CbFut>(&'a self, mut f: F, mut callback: Cb)
    where
        F: (FnMut() -> Fut) + Send,
        Fut: Future + Send,
        Fut::Output: Send,
        Cb: (FnMut(&'a Self, Fut::Output) -> CbFut) + Send,
        CbFut: Future,
        T: Send + 'static,
    {
        loop {
            let ret = f().await;
            callback(self, ret).await;
        }
    }

    pub async fn on3<'a, F, Fut, Cb, CbFut>(&'a self, mut f: F, mut callback: Cb)
    where
        F: (FnMut() -> Fut) + Send,
        Fut: Future + Send,
        Fut::Output: Send,
        Cb: (FnMut(&'a Self, &mut T, Fut::Output) -> CbFut) + Send,
        CbFut: Future,
        T: Send + 'static,
    {
        let Some(app) = self.app.upgrade()
        else {
            return;
        };

        loop {
            let ret = f().await;
            let mut guard = app.lock().await;
            callback(self, &mut guard, ret).await;
        }
    }
}
