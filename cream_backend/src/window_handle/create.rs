use std::sync::Arc;

use crate::{
    runtime::{global::WindowRegiterMutex, rt_event::WindowReg},
    AppWindow,
};

use anyhow::Result;
use tokio::sync::{oneshot, Mutex};
use winit::window::WindowBuilder;

use super::{close_handle::CloseHandle, WindowHandle};

impl<A: AppWindow> WindowHandle<A> {
    pub async fn create<F>(f: F) -> Result<Self>
    where
        F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
    {
        let (window_giver, window_receiver) = oneshot::channel();

        WindowRegiterMutex::lock()
            .await
            .send(WindowReg::RawWindowRequest {
                builder: Box::new(f),
                window_giver,
            });

        let raw_window = Arc::new(
            window_receiver
                .await
                .expect("inner error: cannot receive window initializing result from runtime")?,
        );

        let app = Arc::new(Mutex::new(A::on_create(
            &raw_window,
            CloseHandle(raw_window.id()),
        )?));

        WindowRegiterMutex::lock()
            .await
            .send(WindowReg::WindowRegister {
                app: app.clone() as _,
                raw_window: raw_window.clone(),
            });

        Ok(WindowHandle { app, raw_window })
    }
}
