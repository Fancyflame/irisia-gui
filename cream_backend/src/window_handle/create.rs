use std::sync::Arc;

use crate::{
    runtime::{global::WindowRegiterMutex, rt_event::WindowReg},
    AppWindow,
};

use anyhow::Result;
use tokio::sync::{oneshot, Mutex};
use winit::window::WindowBuilder;

use super::{close_handle::CloseHandle, WindowHandle};

impl WindowHandle {
    pub async fn create<A, F>(f: F) -> Result<Self>
    where
        A: AppWindow,
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

        let raw_window_cloned = raw_window.clone();
        let app = move || {
            let window_id = raw_window_cloned.id();
            Arc::new(Mutex::new(A::on_create(
                raw_window_cloned,
                CloseHandle(window_id),
            ))) as Arc<Mutex<dyn AppWindow>>
        };

        WindowRegiterMutex::lock()
            .await
            .send(WindowReg::WindowRegister {
                app: Box::new(app) as _,
                raw_window: raw_window.clone(),
            });

        Ok(WindowHandle { raw_window })
    }
}
