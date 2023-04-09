use std::sync::Arc;

use tokio::sync::{oneshot, Mutex};
use winit::{
    error::OsError,
    window::{WindowBuilder, WindowId},
};

use crate::{AppWindow, WinitWindow};

pub(crate) enum WindowReg {
    RawWindowRequest {
        builder: Box<dyn FnOnce(WindowBuilder) -> WindowBuilder + Send>,
        window_giver: oneshot::Sender<Result<WinitWindow, OsError>>,
    },

    WindowRegister {
        app: Box<dyn FnOnce() -> Arc<Mutex<dyn AppWindow>> + Send>,
        raw_window: Arc<WinitWindow>,
    },

    WindowDestroyed(WindowId),

    Exit(i32),
}
