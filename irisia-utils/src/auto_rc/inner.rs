use std::{
    cell::Cell,
    mem::ManuallyDrop,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
    thread::ThreadId,
};

const MAX_ATOMIC_REF: usize = isize::MAX as _;

struct SharedCount {
    origin_thread_id: ThreadId,
    strong: AtomicUsize,
    weak: AtomicUsize,
}

pub(super) struct RcInner<T: ?Sized> {
    local_strong: Cell<usize>,
    local_weak: Cell<usize>,
    shared: Option<SharedCount>,
    data: ManuallyDrop<T>,
}

fn abort() -> usize {
    std::process::abort()
}

impl<T: ?Sized> RcInner<T> {
    pub fn new(value: T) -> *mut Self
    where
        T: Sized,
    {
        Box::into_raw(Box::new(Self {
            local_strong: Cell::new(1),
            local_weak: Cell::new(0),
            shared: None,
            data: ManuallyDrop::new(value),
        }))
    }

    pub unsafe fn get_data<'a>(this: *mut Self) -> &'a T {
        &(*this).data
    }

    unsafe fn drop_self(this: *mut Self) {
        drop(Box::from_raw(this));
    }

    unsafe fn drop_data(this: *mut Self) {
        ManuallyDrop::drop(&mut (*this).data);
    }

    // Rc

    pub fn clone_rc(&self) {
        self.local_strong.set(
            self.local_strong
                .take()
                .checked_add(1)
                .unwrap_or_else(abort),
        );
    }

    pub unsafe fn drop_rc(this: *mut Self) {
        if (*this).local_strong.replace(
            (*this)
                .local_strong
                .take()
                .checked_sub(1)
                .unwrap_or_else(abort),
        ) != 1
        {
            return;
        }

        match &(*this).shared {
            Some(shared) => Self::drop_arc(this),
            None => {
                Self::drop_data(this);
                if (*this).local_weak.take() == 0 {
                    Self::drop_self(this);
                }
            }
        }
    }

    // Weak

    pub unsafe fn downgrade_rc(this: *mut Self) {
        if (*this).local_weak.replace(
            (*this)
                .local_weak
                .take()
                .checked_add(1)
                .unwrap_or_else(abort),
        ) != 0
        {
            return;
        }

        if let Some(shared) = &(*this).shared {
            Self::clone_aweak(this);
        }
    }

    pub fn clone_weak(&self) {
        self.local_weak
            .set(self.local_weak.take().checked_add(1).unwrap_or_else(abort));
    }

    pub fn upgrade_weak(&self) -> bool {
        match &self.shared {
            Some(s) => s
                .strong
                .fetch_update(std::sync::atomic::Ordering::Acquire, Relaxed, |prv| {
                    if prv == 0 {
                        None
                    } else {
                        Some(prv + 1)
                    }
                })
                .is_ok(),
            None => {
                if self.local_strong.take() != 0 {
                    self.clone_rc();
                    true
                } else {
                    false
                }
            }
        }
    }

    pub unsafe fn drop_weak(this: *mut Self) {
        if (*this).local_weak.replace(
            (*this)
                .local_weak
                .take()
                .checked_sub(1)
                .unwrap_or_else(abort),
        ) != 1
        {
            return;
        }

        match &(*this).shared {
            Some(shared) => Self::drop_aweak(this),
            None => {
                if (*this).local_strong.take() == 0 {
                    Self::drop_self(this);
                }
            }
        }
    }

    // Arc

    pub unsafe fn upgrade_rc(this: *mut Self) {
        match &(*this).shared {
            Some(SharedCount { strong, .. }) => {
                Self::clone_arc(this);
            }
            None => {
                (*this).shared = Some(SharedCount {
                    origin_thread_id: std::thread::current().id(),

                    // one for all Rc, one for that new Arc
                    strong: AtomicUsize::new(2),

                    weak: AtomicUsize::new(if (*this).local_weak.take() != 0 { 1 } else { 0 }),
                });
            }
        }
    }

    unsafe fn get_shared_unchecked<'a>(this: *mut Self) -> &'a SharedCount {
        (*this).shared.as_ref().unwrap_unchecked()
    }

    pub unsafe fn clone_arc(this: *mut Self) {
        let shared = Self::get_shared_unchecked(this);
        if shared.strong.fetch_add(1, Relaxed) == MAX_ATOMIC_REF {
            abort();
        }
    }

    pub unsafe fn drop_arc(this: *mut Self) {
        let shared = Self::get_shared_unchecked(this);
        if shared.strong.fetch_sub(1, Relaxed) != 1 {
            return;
        }

        Self::drop_data(this);
        if shared.weak.load(Relaxed) == 0 {
            Self::drop_self(this);
        }
    }

    // AWeak

    pub unsafe fn downgrade_arc(this: *mut Self) {
        Self::clone_aweak(this);
    }

    pub unsafe fn clone_aweak(this: *mut Self) {
        let shared = Self::get_shared_unchecked(this);
        if shared.weak.fetch_add(1, Relaxed) == MAX_ATOMIC_REF {
            abort();
        }
    }

    pub unsafe fn drop_aweak(this: *mut Self) {
        let shared = Self::get_shared_unchecked(this);
        if shared.weak.fetch_sub(1, Relaxed) != 1 {
            return;
        }

        if shared.strong.load(Relaxed) == 0 {
            Self::drop_self(this);
        }
    }
}
