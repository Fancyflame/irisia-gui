use std::sync::Arc;

use tokio::sync::oneshot;
use winit::{
    error::OsError,
    window::{WindowBuilder, WindowId},
};

use crate::{AppWindow, WinitWindow};

pub(crate) type AppBuildFn = Box<dyn FnOnce() -> Box<dyn AppWindow> + Send>;

pub(crate) enum WindowReg {
    RawWindowRequest {
        builder: WindowBuilder,
        window_giver: oneshot::Sender<Result<WinitWindow, OsError>>,
    },

    WindowRegister {
        app: AppBuildFn,
        raw_window: Arc<WinitWindow>,
    },

    WindowDestroyed(WindowId),

    Exit,
}
