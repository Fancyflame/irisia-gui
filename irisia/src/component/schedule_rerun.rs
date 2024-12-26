use std::{cell::RefCell, collections::VecDeque, rc::Weak};

use super::{CompModelInner, Component};

pub(super) trait RerunNode {
    fn set_needs_rerun(&mut self) -> bool;
    fn rerun_checked(&mut self);
}

impl<Pr> RerunNode for CompModelInner<Pr>
where
    Pr: Component,
{
    fn set_needs_rerun(&mut self) -> bool {
        let prev = self.needs_rerun;
        self.needs_rerun = true;
        prev
    }

    fn rerun_checked(&mut self) {
        if self.needs_rerun {
            self.rerun();
        }
    }
}

struct Queue {
    working: bool,
    tasks: VecDeque<ScheduleRerun>,
}

#[derive(Clone)]
pub struct ScheduleRerun(pub(super) Weak<RefCell<dyn RerunNode>>);

impl ScheduleRerun {
    pub fn schedule(&self) {
        let Some(rc) = self.0.upgrade() else {
            return;
        };

        // if the task is running, then the update is emitted by itself,
        // just ignore it
        let Ok(mut inner) = rc.try_borrow_mut() else {
            return;
        };

        if !inner.set_needs_rerun() {
            push_task(self);
        }
    }

    fn rerun(&self) {
        if let Some(rc) = self.0.upgrade() {
            rc.borrow_mut().rerun_checked();
        }
    }
}

fn push_task(insertion: &ScheduleRerun) {
    thread_local! {
        static QUEUE: RefCell<Queue> = RefCell::new(Queue {
            working: false,
            tasks: VecDeque::new(),
        });
    }

    let already_has_worker = QUEUE.with_borrow_mut(|queue| {
        let exit = !queue.working;
        queue.tasks.push_back(insertion.clone());
        queue.working = true;
        exit
    });

    if already_has_worker {
        return;
    }

    while let Some(task) = QUEUE.with_borrow_mut(|queue| queue.tasks.pop_front()) {
        task.rerun();
    }
    QUEUE.with_borrow_mut(|queue| {
        queue.working = false;
    });
}
