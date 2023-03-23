use crate::event::Event;
use cream_backend::WindowEvent;

use crate::event::EventEmitter;

pub(super) async fn _emit_window_event(event_emitter: &EventEmitter, event: WindowEvent) {
    event_emitter.emit(&event).await;
}

impl Event for WindowEvent {}

/*macro_rules! make_events {
    ()=>{};
    (
        $Struct:ident,
        $($rest:tt)*
    ) => {
        pub struct $Struct;
        impl Event for $Struct {}
        make_events!($($rest)*);
    };
    (
        $Struct:ident($($tt:tt)*),
        $($rest:tt)*
    ) => {
        pub struct $Struct($($tt)*);
        impl Event for $Struct {}
        make_events!($($rest)*);
    };
    (
        $Struct:ident{$($tt:tt)*},
        $($rest:tt)*
    ) => {
        pub struct $Struct {
            $($tt)*
        }
        impl Event for $Struct {}
        make_events!($($rest)*);
    };
}
*/
