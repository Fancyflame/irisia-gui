use super::ElementHandle;

#[must_use]
pub struct Listen<'a, Wv, Io, Rs> {
    pub(super) eh: &'a ElementHandle,
    pub(super) with_value: Wv,
    pub(super) is_once: Io,
    pub(super) recv_sys: Rs,
}

pub struct WithValue<T>(pub(super) T);
pub struct IsOnce;
pub struct RecvSys;

impl<'a> Listen<'a, (), (), ()> {
    pub(crate) fn new(eh: &'a ElementHandle) -> Self {
        Listen {
            eh,
            with_value: (),
            is_once: (),
            recv_sys: (),
        }
    }
}

impl<'a, Wv, Io, Rs> Listen<'a, Wv, Io, Rs> {
    pub fn with_value<T>(self, value: T) -> Listen<'a, WithValue<T>, Io, Rs>
    where
        T: Send + 'static,
    {
        Listen {
            eh: self.eh,
            with_value: WithValue(value),
            is_once: self.is_once,
            recv_sys: self.recv_sys,
        }
    }

    pub fn set_once(self) -> Listen<'a, Wv, IsOnce, Rs> {
        Listen {
            eh: self.eh,
            with_value: self.with_value,
            is_once: IsOnce,
            recv_sys: self.recv_sys,
        }
    }

    pub fn recv_sys(self) -> Listen<'a, Wv, Io, RecvSys> {
        Listen {
            eh: self.eh,
            with_value: self.with_value,
            is_once: self.is_once,
            recv_sys: RecvSys,
        }
    }
}
