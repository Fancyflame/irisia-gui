use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::runtime::{global::WindowRegiterMutex, rt_event::WindowReg};

pub struct Timer {
    id_counter: u64,
    time_points: VecDeque<TimeWatcher>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            id_counter: 1, // cannot be 0, see `set_timeout_at`
            time_points: VecDeque::new(),
        }
    }

    pub fn update(&mut self, time: Instant) {
        let point = self
            .time_points
            .partition_point(|TimeWatcher { instant, .. }| time >= *instant);

        for _ in 0..point {
            let TimeWatcher {
                instant,
                inner,
                cancel_handle,
            } = self.time_points.pop_front().unwrap();

            if cancel_handle.is_none() {
                continue;
            }

            match inner {
                TimeWatcherInner::SetTimeout { callback } => callback(),
                TimeWatcherInner::SetInterval {
                    interval,
                    mut callback,
                } => {
                    callback();
                    let new_instant = instant + interval;
                    WindowRegiterMutex::lock().send(WindowReg::SetWait(new_instant));
                    self.time_points.push_back(TimeWatcher {
                        instant: new_instant,
                        inner: TimeWatcherInner::SetInterval { interval, callback },
                        cancel_handle,
                    })
                }
            }
        }
    }

    pub fn set_timeout<F>(&mut self, f: F, timeout: Duration) -> CancelHandle
    where
        F: FnOnce() + 'static,
    {
        self.set_timeout_at(f, Instant::now() + timeout)
    }

    pub fn set_timeout_at<F>(&mut self, f: F, instant: Instant) -> CancelHandle
    where
        F: FnOnce() + 'static,
    {
        if instant < Instant::now() {
            f();
            return CancelHandle(0);
        }

        self.register_timeout(
            instant,
            TimeWatcherInner::SetTimeout {
                callback: Box::new(f),
            },
        )
    }

    pub fn set_interval<F>(&mut self, f: F, interval: Duration) -> CancelHandle
    where
        F: FnMut() + 'static,
    {
        self.register_timeout(
            Instant::now() + interval,
            TimeWatcherInner::SetInterval {
                interval,
                callback: Box::new(f),
            },
        )
    }

    pub fn cancel(&mut self, handle: CancelHandle) -> bool {
        let h = Some(handle.0);
        for watcher in &mut self.time_points {
            if watcher.cancel_handle == h {
                watcher.cancel_handle.take();
                return true;
            }
        }
        false
    }

    fn register_timeout(&mut self, instant: Instant, watcher: TimeWatcherInner) -> CancelHandle {
        WindowRegiterMutex::lock().send(WindowReg::SetWait(instant));

        let id = self.id_counter;
        assert_ne!(id, u64::MAX);
        self.id_counter += 1;

        self.time_points.push_back(TimeWatcher {
            instant,
            inner: watcher,
            cancel_handle: Some(id),
        });

        CancelHandle(id)
    }
}

#[derive(Clone, Copy)]
pub struct CancelHandle(u64);

struct TimeWatcher {
    instant: Instant,
    inner: TimeWatcherInner,
    cancel_handle: Option<u64>,
}

enum TimeWatcherInner {
    SetTimeout {
        callback: Box<dyn FnOnce()>,
    },
    SetInterval {
        interval: Duration,
        callback: Box<dyn FnMut()>,
    },
}
