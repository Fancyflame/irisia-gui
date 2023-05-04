use super::ElementHandle;

pub struct On<'a, Wv, Io, Rso> {
    pub(super) eh: &'a ElementHandle,
    pub(super) with_value: Wv,
    pub(super) is_once: Io,
    pub(super) recv_sys_only: Rso,
}

pub struct WithValue<T>(pub(super) T);
pub struct IsOnce;
pub struct RecvSysOnly;

impl<'a> On<'a, (), (), ()> {
    pub(crate) fn new(eh: &'a ElementHandle) -> Self {
        On {
            eh,
            with_value: (),
            is_once: (),
            recv_sys_only: (),
        }
    }
}

impl<'a, Wv, Io, Rso> On<'a, Wv, Io, Rso> {
    pub fn with_value<T>(self, value: T) -> On<'a, WithValue<T>, Io, Rso>
    where
        T: Send + 'static,
    {
        On {
            eh: self.eh,
            with_value: WithValue(value),
            is_once: self.is_once,
            recv_sys_only: self.recv_sys_only,
        }
    }

    pub fn set_once(self) -> On<'a, Wv, IsOnce, Rso> {
        On {
            eh: self.eh,
            with_value: self.with_value,
            is_once: IsOnce,
            recv_sys_only: self.recv_sys_only,
        }
    }

    pub fn recv_sys_only(self) -> On<'a, Wv, Io, RecvSysOnly> {
        On {
            eh: self.eh,
            with_value: self.with_value,
            is_once: self.is_once,
            recv_sys_only: RecvSysOnly,
        }
    }
}
