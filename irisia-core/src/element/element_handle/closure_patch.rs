use tokio::task::JoinHandle;

use crate::{event::EventMetadata, Event};

use super::callback::{IsOnce, On, RecvSysOnly, WithValue};

impl On<'_, (), (), ()> {
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

impl<T> On<'_, WithValue<T>, (), ()>
where
    T: Send + 'static,
{
    pub fn spawn<E, F>(mut self, mut f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnMut(E, EventMetadata, &mut T) + Send + 'static,
    {
        On::new(self.eh).spawn(move |ev: E, em| {
            f(ev, em, &mut self.with_value.0);
        })
    }
}

impl On<'_, (), IsOnce, ()> {
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

impl<T> On<'_, WithValue<T>, IsOnce, ()>
where
    T: Send + 'static,
{
    pub fn spawn<E, F>(self, f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnOnce(E, EventMetadata, T) + Send + 'static,
    {
        On::new(self.eh).set_once().spawn(move |ev: E, em| {
            f(ev, em, self.with_value.0);
        })
    }
}

impl On<'_, (), (), RecvSysOnly> {
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

impl<T> On<'_, WithValue<T>, (), RecvSysOnly>
where
    T: Send + 'static,
{
    pub fn spawn<E, F>(mut self, mut f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnMut(E, &mut T) + Send + 'static,
    {
        On::new(self.eh).recv_sys_only().spawn(move |ev: E| {
            f(ev, &mut self.with_value.0);
        })
    }
}

impl On<'_, (), IsOnce, RecvSysOnly> {
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

impl<T> On<'_, WithValue<T>, IsOnce, RecvSysOnly>
where
    T: Send + 'static,
{
    pub fn spawn<E, F>(self, f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnOnce(E, T) + Send + 'static,
    {
        On::new(self.eh)
            .recv_sys_only()
            .set_once()
            .spawn(move |ev: E| {
                f(ev, self.with_value.0);
            })
    }
}
