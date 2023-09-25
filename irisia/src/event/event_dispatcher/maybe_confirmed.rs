use std::sync::atomic::AtomicUsize;

use tokio::sync::{Semaphore, TryAcquireError};

// only when all permits taken back can emit events
pub(crate) struct MaybeConfirmed {
    confirmed: Semaphore,
    defer_cancel: AtomicUsize,
}

const MAX_LOCKS: usize = 1000000;

impl MaybeConfirmed {
    pub fn new() -> Self {
        MaybeConfirmed {
            confirmed: Semaphore::new(MAX_LOCKS),
            defer_cancel: AtomicUsize::new(0),
        }
    }

    pub fn confirm_one(&self) {
        self.confirmed.add_permits(1);
    }

    pub fn cancel_one(&self) {
        self.cancel_many(1);
    }

    pub fn cancel_many(&self, count: usize) {
        match self.confirmed.try_acquire_many(count as _) {
            Ok(permit) => permit.forget(),
            Err(_) => {
                self.defer_cancel
                    .fetch_add(count, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    pub async fn all_confirmed(&self) -> AllConfirmedPermits {
        loop {
            self.confirmed
                .acquire_many(MAX_LOCKS as _)
                .await
                .unwrap()
                .forget();

            let defer_cancel_count = self
                .defer_cancel
                .swap(0, std::sync::atomic::Ordering::Relaxed);

            if defer_cancel_count == 0 {
                break;
            }

            self.confirmed
                .add_permits((MAX_LOCKS - defer_cancel_count) as _);
        }

        AllConfirmedPermits {
            maybe_confirmed: self,
            semaphore_permits: MAX_LOCKS,
        }
    }

    pub fn try_all_confirmed(&self) -> Result<AllConfirmedPermits, TryAcquireError> {
        self.confirmed.try_acquire_many(MAX_LOCKS as _)?.forget();

        let defer_cancel_count = self
            .defer_cancel
            .swap(0, std::sync::atomic::Ordering::Relaxed);

        if defer_cancel_count != 0 {
            self.confirmed
                .add_permits((MAX_LOCKS - defer_cancel_count) as _);
            return Err(TryAcquireError::NoPermits);
        }

        Ok(AllConfirmedPermits {
            maybe_confirmed: self,
            semaphore_permits: MAX_LOCKS,
        })
    }
}

pub struct AllConfirmedPermits<'a> {
    maybe_confirmed: &'a MaybeConfirmed,
    semaphore_permits: usize,
}

impl AllConfirmedPermits<'_> {
    pub fn cancel_many(&mut self, count: usize) {
        assert_ne!(count, MAX_LOCKS);
        self.semaphore_permits -= count;
    }
}

impl Drop for AllConfirmedPermits<'_> {
    fn drop(&mut self) {
        self.maybe_confirmed
            .confirmed
            .add_permits(self.semaphore_permits as _);
    }
}
