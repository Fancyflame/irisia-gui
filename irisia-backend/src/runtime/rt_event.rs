use std::{fmt::Debug, sync::Arc};

use tokio::sync::oneshot;
use winit::{
    error::OsError,
    window::{WindowAttributes, WindowId},
};

use crate::{AppWindow, WinitWindow};

pub(crate) type AppBuildFn = Box<dyn FnOnce() -> Box<dyn AppWindow> + Send>;

pub(crate) enum WindowReg {
    RawWindowRequest {
        window_attributes: WindowAttributes,
        window_giver: oneshot::Sender<Result<WinitWindow, OsError>>,
    },

    WindowRegister {
        app: AppBuildFn,
        raw_window: Arc<WinitWindow>,
    },

    WindowDestroyed(WindowId),

    Exit,
}

impl Debug for WindowReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowReg::RawWindowRequest { .. } => f.debug_struct("RawWindowRequest").finish(),
            WindowReg::WindowRegister { .. } => f.debug_struct("WindowRegister").finish(),
            WindowReg::WindowDestroyed(..) => f.debug_tuple("WindowDestroyed").finish(),
            WindowReg::Exit => f.write_str("Exit"),
        }
    }
}
