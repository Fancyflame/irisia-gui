use crate as cream_core;
use crate::{event::Event, primary::Point};
use cream_backend::WindowEvent;
use cream_macros::Event;
impl Event for WindowEvent {}

#[derive(Event, Clone)]
pub struct PointerDown {
    pub position: Point,
}

#[derive(Event, Clone)]
pub struct PointerUp {
    pub position: Point,
}

#[derive(Event, Clone)]
pub struct PointerMove {
    pub position: Point,
}

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
