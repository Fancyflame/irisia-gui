use std::{any::TypeId, rc::Rc};

use anyhow::{anyhow, Result};

use super::event_flow::EventFlow;

pub trait IntoCallback<T> {
    fn call_full(&self, value: &T, flow: &mut EventFlow);
}

impl<F, T> IntoCallback<T> for F
where
    F: Fn(&T, &mut EventFlow),
{
    fn call_full(&self, value: &T, flow: &mut EventFlow) {
        self(value, flow);
    }
}

impl<T> IntoCallback<T> for fn(&T) {
    fn call_full(&self, value: &T, _: &mut EventFlow) {
        self(value)
    }
}

pub(super) struct AnonymousCallback {
    call_body: *mut dyn IntoCallback<()>,
    type_id: TypeId,
}

impl AnonymousCallback {
    pub fn new<T: 'static, I>(cb: Rc<I>) -> Self
    where
        I: IntoCallback<T>,
    {
        AnonymousCallback {
            call_body: Rc::into_raw(cb as Rc<dyn IntoCallback<T>>) as *mut dyn IntoCallback<T> as _,
            type_id: TypeId::of::<T>(),
        }
    }

    pub fn try_call<T: 'static>(&self, value: &T, flow: &mut EventFlow) -> Result<()> {
        if TypeId::of::<T>() != self.type_id {
            return Err(anyhow!("Type didn't match"));
        }

        let cb = unsafe { &*(self.call_body as *mut dyn IntoCallback<T>) };
        cb.call_full(value, flow);
        Ok(())
    }
}

impl Drop for AnonymousCallback {
    fn drop(&mut self) {
        unsafe {
            Rc::from_raw(self.call_body);
        }
    }
}
