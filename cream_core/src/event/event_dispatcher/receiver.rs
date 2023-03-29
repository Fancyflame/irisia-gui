use std::{future::Future, marker::PhantomData, task::Poll};

use crate::Event;

use super::EventDispatcher;

pub struct EventReceive<'ed, E, K>
where
    E: Event,
    K: Clone + Send + 'static,
{
    _phantom: PhantomData<(E, K)>,
    dispatcher: &'ed EventDispatcher,
    id: u32,
    taken: bool,
}

impl<'ed, E, K> EventReceive<'ed, E, K>
where
    E: Event,
    K: Clone + Send + 'static,
{
    pub fn new(dispatcher: &'ed EventDispatcher, id: u32) -> Self {
        EventReceive {
            _phantom: PhantomData,
            dispatcher,
            id,
            taken: false,
        }
    }
}

impl<E, K> Future for EventReceive<'_, E, K>
where
    E: Event,
    K: Clone + Send + Unpin + 'static,
{
    type Output = (E, K);
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.dispatcher.0.lock().unwrap();
        let item = guard.get_item();

        match item.take(self.id) {
            Some(pair) => {
                drop(guard);
                self.get_mut().taken = true;
                Poll::Ready(pair)
            }
            None => {
                item.update_waker(self.id, cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<E, K> Drop for EventReceive<'_, E, K>
where
    E: Event,
    K: Clone + Send + 'static,
{
    fn drop(&mut self) {
        if !self.taken {
            self.dispatcher
                .0
                .lock()
                .unwrap()
                .get_item::<E, K>()
                .clear_by_id(self.id);
        }
    }
}
