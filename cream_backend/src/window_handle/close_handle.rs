use winit::window::WindowId;

use crate::runtime::{global::WindowRegiterMutex, rt_event::WindowReg};

#[derive(Clone)]
pub struct CloseHandle(pub(super) WindowId);

impl CloseHandle {
    pub fn close(&self) {
        let window_id = self.0;
        tokio::spawn(async move {
            WindowRegiterMutex::lock()
                .await
                .send(WindowReg::WindowDestroyed(window_id));
        });
    }
}
