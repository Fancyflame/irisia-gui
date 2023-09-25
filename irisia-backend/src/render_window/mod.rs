use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::{sync::mpsc, task::LocalSet};
use winit::event::Event;

use crate::{runtime::rt_event::AppBuildFn, WinitWindow};

use self::window::RenderWindow;

mod renderer;
mod window;

enum Command {
    Redraw,
    HandleEvent(Event<'static, ()>),
}

pub struct RenderWindowController {
    chan: mpsc::UnboundedSender<Command>,
}

impl RenderWindowController {
    pub fn new(app: AppBuildFn, window: Arc<WinitWindow>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Command>();

        std::thread::Builder::new()
            .name("irisia window".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
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
                            Command::Redraw => rw.redraw(),
                            Command::HandleEvent(ev) => rw.handle_event(ev),
                        }
                    }
                });
                rt.block_on(local);
            })
            .unwrap();

        Self { chan: tx }
    }

    pub fn redraw(&self) -> Result<()> {
        self.chan
            .send(Command::Redraw)
            .map_err(|_| recv_shut_down_error())
    }

    pub fn handle_event(&self, event: Event<'static, ()>) -> Result<()> {
        self.chan
            .send(Command::HandleEvent(event))
            .map_err(|_| recv_shut_down_error())
    }
}

fn recv_shut_down_error() -> anyhow::Error {
    anyhow!("worker unexpectedly shutted down")
}
