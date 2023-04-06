use std::{future::Future, marker::PhantomData, task::Poll};

use crate::{event::EventMetadata, Event};

use super::EventDispatcher;

pub struct EventReceive<'ed, E: Event> {
    _phantom: PhantomData<E>,
    dispatcher: &'ed EventDispatcher,
    id: u32,
    taken: bool,
}

impl<'ed, E: Event> EventReceive<'ed, E> {
    pub fn new(dispatcher: &'ed EventDispatcher, id: u32) -> Self {
        EventReceive {
            _phantom: PhantomData,
            dispatcher,
            id,
            taken: false,
        }
    }
}

impl<E: Event> Future for EventReceive<'_, E> {
    type Output = (E, EventMetadata);
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        assert!(!self.taken);

        match self
            .dispatcher
            .0
            .lock()
            .unwrap()
            .item_map
            .get_exist::<E>()
            .poll(self.id, cx.waker().clone())
        {
            Some(pair) => {
                self.get_mut().taken = true;
                Poll::Ready(pair)
            }

            None => Poll::Pending,
        }
    }
}

impl<E: Event> Drop for EventReceive<'_, E> {
    fn drop(&mut self) {
        if !self.taken {
            self.dispatcher
                .0
                .lock()
                .unwrap()
                .item_map
                .get_exist::<E>()
                .clear_by_id(self.id);
        }
    }
}
