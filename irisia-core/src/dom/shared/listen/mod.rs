use super::ElementHandle;

mod implement;

pub struct Listen<'a, El, T0, T1, T2, T3> {
    eh: &'a ElementHandle<El>,
    once: T0,
    sys_only: T1,
    without_handle: T2,
    sub_event: T3,
}

pub struct FlagSet;

impl<'a, El> Listen<'a, El, (), (), (), ()> {
    pub(super) fn new(eh: &'a ElementHandle<El>) -> Self {
        Listen {
            eh,
            once: (),
            sys_only: (),
            without_handle: (),
            sub_event: (),
        }
    }
}

impl<'a, El, T0, T1, T2, T3> Listen<'a, El, T0, T1, T2, T3> {
    pub fn once(self) -> Listen<'a, El, FlagSet, T1, T2, T3> {
        Listen {
            eh: self.eh,
            once: FlagSet,
            sys_only: self.sys_only,
            without_handle: self.without_handle,
            sub_event: self.sub_event,
        }
    }

    pub fn sys_only(self) -> Listen<'a, El, T0, FlagSet, T2, T3> {
        Listen {
            eh: self.eh,
            once: self.once,
            sys_only: FlagSet,
            without_handle: self.without_handle,
            sub_event: self.sub_event,
        }
    }

    pub fn without_handle(self) -> Listen<'a, El, T0, T1, FlagSet, T3> {
        Listen {
            eh: self.eh,
            once: self.once,
            sys_only: self.sys_only,
            without_handle: FlagSet,
            sub_event: self.sub_event,
        }
    }

    pub fn sub_event(self) -> Listen<'a, El, T0, T1, T2, FlagSet> {
        Listen {
            eh: self.eh,
            once: self.once,
            sys_only: self.sys_only,
            without_handle: self.without_handle,
            sub_event: FlagSet,
        }
    }
}
