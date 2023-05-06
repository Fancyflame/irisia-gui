use tokio::task::JoinHandle;

use crate::{event::EventMetadata, Event};

use super::callback::{IsOnce, Listen, RecvSys, WithValue};

impl Listen<'_, (), (), ()> {
    pub fn spawn<E, F>(self, mut f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnMut(E, EventMetadata) + Send + 'static,
    {
        let ed = self.eh.event_dispatcher().clone();
        tokio::spawn(async move {
            let mut lock = ed.lock();
            ed.cancel_on_abandoned(async {
                loop {
                    let recv = lock.recv::<E>().await;
                    f(recv.0, recv.1);
                }
            })
            .await;
        })
    }
}

impl<T> Listen<'_, WithValue<T>, (), ()>
where
    T: Send + 'static,
{
    pub fn spawn<E, F>(mut self, mut f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnMut(E, EventMetadata, &mut T) + Send + 'static,
    {
        Listen::new(self.eh).spawn(move |ev: E, em| {
            f(ev, em, &mut self.with_value.0);
        })
    }
}

impl Listen<'_, (), IsOnce, ()> {
    pub fn spawn<E, F>(self, f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnOnce(E, EventMetadata) + Send + 'static,
    {
        let ed = self.eh.event_dispatcher().clone();
        tokio::spawn(async move {
            ed.cancel_on_abandoned(async {
                let recv = ed.recv::<E>().await;
                f(recv.0, recv.1);
            })
            .await;
        })
    }
}

impl<T> Listen<'_, WithValue<T>, IsOnce, ()>
where
    T: Send + 'static,
{
    pub fn spawn<E, F>(self, f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnOnce(E, EventMetadata, T) + Send + 'static,
    {
        Listen::new(self.eh).set_once().spawn(move |ev: E, em| {
            f(ev, em, self.with_value.0);
        })
    }
}

impl Listen<'_, (), (), RecvSys> {
    pub fn spawn<E, F>(self, mut f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnMut(E) + Send + 'static,
    {
        let ed = self.eh.event_dispatcher().clone();
        tokio::spawn(async move {
            let mut lock = ed.lock();
            ed.cancel_on_abandoned(async {
                loop {
                    f(lock.recv_sys::<E>().await);
                }
            })
            .await;
        })
    }
}

impl<T> Listen<'_, WithValue<T>, (), RecvSys>
where
    T: Send + 'static,
{
    pub fn spawn<E, F>(mut self, mut f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnMut(E, &mut T) + Send + 'static,
    {
        Listen::new(self.eh).recv_sys().spawn(move |ev: E| {
            f(ev, &mut self.with_value.0);
        })
    }
}

impl Listen<'_, (), IsOnce, RecvSys> {
    pub fn spawn<E, F>(self, f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnOnce(E) + Send + 'static,
    {
        let ed = self.eh.event_dispatcher().clone();
        tokio::spawn(async move {
            ed.cancel_on_abandoned(async {
                f(ed.recv_sys::<E>().await);
            })
            .await;
        })
    }
}

impl<T> Listen<'_, WithValue<T>, IsOnce, RecvSys>
where
    T: Send + 'static,
{
    pub fn spawn<E, F>(self, f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnOnce(E, T) + Send + 'static,
    {
        Listen::new(self.eh)
            .recv_sys()
            .set_once()
            .spawn(move |ev: E| {
                f(ev, self.with_value.0);
            })
    }
}
