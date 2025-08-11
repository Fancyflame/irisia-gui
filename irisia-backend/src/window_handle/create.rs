use std::sync::Arc;

use crate::{
    runtime::{global_event::WindowRegiterMutex, rt_event::WindowReg},
    AppWindow, WinitWindow,
};

use anyhow::Result;
use tokio::sync::oneshot;
use winit::window::WindowAttributes;

use super::{close_handle::CloseHandle, RawWindowHandle};

impl RawWindowHandle {
    pub async fn create<A, F>(create_app: F, wa: WindowAttributes) -> Result<Self>
    where
        A: AppWindow,
        F: FnOnce(Arc<WinitWindow>, CloseHandle) -> A + Send + 'static,
    {
        let (window_giver, window_receiver) = oneshot::channel();

        WindowRegiterMutex::lock()
            .await
            .send(WindowReg::RawWindowRequest {
                window_attributes: wa,
                window_giver,
            });

        let raw_window = Arc::new(
            window_receiver
                .await?
                .expect("inner error: cannot receive window initializing result from runtime"),
        );

        let raw_window_cloned = raw_window.clone();
        let app = move || {
            let window_id = raw_window_cloned.id();
            Box::new(create_app(raw_window_cloned, CloseHandle(window_id))) as Box<dyn AppWindow>
        };

        WindowRegiterMutex::lock()
            .await
            .send(WindowReg::WindowRegister {
                app: Box::new(app),
                raw_window: raw_window.clone(),
            });

        Ok(RawWindowHandle {
            close_handle: CloseHandle(raw_window.id()),
            raw_window,
        })
    }
}
