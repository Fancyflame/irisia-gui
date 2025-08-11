use std::sync::Arc;

use anyhow::{anyhow, Result};
use pixels::Pixels;
use renderer::Renderer;
use tokio::{sync::mpsc, task::LocalSet};
use winit::event::WindowEvent;

use crate::{runtime::rt_event::AppBuildFn, WinitWindow};

use self::window::RenderWindow;

mod renderer;
mod window;

enum Command {
    Redraw,
    HandleEvent(WindowEvent),
}

pub struct RenderWindowController {
    chan: mpsc::UnboundedSender<Command>,
}

impl RenderWindowController {
    pub fn new(app: AppBuildFn, window: Arc<WinitWindow>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Command>();

        // because some system needs renderer to be created on main thread, like MacOS.
        let pixels = Renderer::create_pixels(&window).expect("cannot create pixels");

        std::thread::Builder::new()
            .name("irisia window".into())
            .spawn(move || window_runtime(app, window, pixels, rx))
            .unwrap();

        Self { chan: tx }
    }

    pub fn redraw(&self) -> Result<()> {
        /*let mut finished = self.draw_finished.lock().unwrap();
        if !*finished {
            return Ok(());
        }

        *finished = false;*/
        self.chan
            .send(Command::Redraw)
            .map_err(|_| recv_shut_down_error())?;

        //while !*finished {
        //    finished = self.draw_finished.1.wait(finished).unwrap();
        //}
        Ok(())
    }

    pub fn handle_event(&self, event: WindowEvent) -> Result<()> {
        self.chan
            .send(Command::HandleEvent(event))
            .map_err(|_| recv_shut_down_error())
    }
}

fn window_runtime(
    app: AppBuildFn,
    window: Arc<WinitWindow>,
    pixels: Pixels,
    mut rx: mpsc::UnboundedReceiver<Command>,
) {
    let async_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let local = LocalSet::new();
    local.block_on(&async_runtime, async move {
        let mut rw =
            RenderWindow::new(app, pixels, window.clone()).expect("cannot launch renderer");

        loop {
            let Some(cmd) = rx.recv().await else {
                break;
            };

            match cmd {
                Command::Redraw => {
                    rw.redraw();
                    //window.request_redraw();
                }
                Command::HandleEvent(ev) => {
                    rw.handle_event(ev);
                }
            }
        }
    });
}

fn recv_shut_down_error() -> anyhow::Error {
    anyhow!("worker unexpectedly shutted down")
}
