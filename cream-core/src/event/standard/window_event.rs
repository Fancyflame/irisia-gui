use std::path::PathBuf;

use crate as cream;
use crate::Event;
use cream_backend::{
    winit::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{
            DeviceId, ElementState, Ime, KeyboardInput, ModifiersState, MouseButton,
            MouseScrollDelta, Touch, TouchPhase,
        },
        window::Theme,
    },
    WindowEvent,
};

impl Event for WindowEvent {}

macro_rules! impl_window_event {
    ()=>{};

    ($Struct:ident ($($tt:tt)*), $($rest:tt)*) => {
        #[derive(Clone, Event)]
        pub struct $Struct($($tt)*);
        impl_window_event!($($rest)*);
    };

    ($Struct:ident {$($tt:tt)*}, $($rest:tt)*) => {
        #[derive(Clone, Event)]
        pub struct $Struct{$($tt)*}
        impl_window_event!($($rest)*);
    };

    ($Struct:ident, $($rest:tt)*) => {
        #[derive(Clone, Event)]
        pub struct $Struct;
        impl_window_event!($($rest)*);
    };
}

impl_window_event! {
    WindowResized(pub PhysicalSize<u32>),
    WindowMoved(pub PhysicalPosition<i32>),
    WindowCloseRequested,
    WindowDestroyed,
    WindowDroppedFile(pub PathBuf),
    WindowHoveredFile(pub PathBuf),
    WindowHoveredFileCancelled,
    WindowReceivedCharacter(pub char),
    WindowFocused(pub bool),
    WindowKeyboardInput {
        pub device_id: DeviceId,
        pub input: KeyboardInput,
        pub is_synthetic: bool,
    },
    WindowModifiersChanged(ModifiersState),
    WindowIme(Ime),
    WindowCursorMoved {
        pub device_id: DeviceId,
        pub position: PhysicalPosition<f64>,
        pub modifiers: ModifiersState,
    },
    WindowCursorEntered {
        pub device_id: DeviceId,
    },
    WindowCursorLeft {
        pub device_id: DeviceId,
    },
    WindowMouseWheel {
        pub device_id: DeviceId,
        pub delta: MouseScrollDelta,
        pub phase: TouchPhase,
        pub modifiers: ModifiersState,
    },
    WindowMouseInput {
        pub device_id: DeviceId,
        pub state: ElementState,
        pub button: MouseButton,
        pub modifiers: ModifiersState,
    },
    WindowTouch(Touch),
    WindowScaleFactorChanged {
        pub scale_factor: f64,
        pub new_inner_size: PhysicalSize<u32>,
    },
    WindowThemeChanged(Theme),
}
