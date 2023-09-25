use std::sync::{Arc, Condvar, Mutex as StdMutex};

use anyhow::{anyhow, Result};
use tokio::{sync::mpsc, task::LocalSet};

use crate::{runtime::rt_event::AppBuildFn, StaticWindowEvent, WinitWindow};

use self::window::RenderWindow;

mod renderer;
mod window;

enum Command {
    Redraw,
    HandleEvent(StaticWindowEvent),
}

pub struct RenderWindowController {
    chan: mpsc::UnboundedSender<Command>,
    draw_finished: Arc<(StdMutex<bool>, Condvar)>,
}

impl RenderWindowController {
    pub fn new(app: AppBuildFn, window: Arc<WinitWindow>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Command>();
        let draw_finished = Arc::new((StdMutex::new(true), Condvar::new()));
        let draw_finished_cloned = draw_finished.clone();

        std::thread::Builder::new()
            .name("irisia window".into())
            .spawn(move || {
                let async_runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                let local = LocalSet::new();
                local.spawn_local(async move {
                    let mut rw = RenderWindow::new(app, window).expect("cannot launch renderer");
                    loop {
                        let Some(cmd) = rx.recv().await
                        else {
                            return;
                        };

                        match cmd {
                            Command::Redraw => {
                                rw.redraw();
                                *draw_finished.0.lock().unwrap() = true;
                                draw_finished.1.notify_all();
                            }
                            Command::HandleEvent(ev) => rw.handle_event(ev),
                        }
                    }
                });
                async_runtime.block_on(local);
            })
            .unwrap();

        Self {
            chan: tx,
            draw_finished: draw_finished_cloned,
        }
    }

    pub fn redraw(&self) -> Result<()> {
        let mut finished = self.draw_finished.0.lock().unwrap();
        *finished = false;
        self.chan
            .send(Command::Redraw)
            .map_err(|_| recv_shut_down_error())?;

        while !*finished {
            finished = self.draw_finished.1.wait(finished).unwrap();
        }
        Ok(())
    }

    pub fn handle_event(&self, event: StaticWindowEvent) -> Result<()> {
        self.chan
            .send(Command::HandleEvent(event))
            .map_err(|_| recv_shut_down_error())
    }
}

fn recv_shut_down_error() -> anyhow::Error {
    anyhow!("worker unexpectedly shutted down")
}
