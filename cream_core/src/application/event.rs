use crate as cream_core;
use crate::{event::Event, primary::Point};
use cream_backend::winit::dpi::PhysicalPosition;
use cream_backend::WindowEvent;
use cream_macros::Event;

use super::elem_table::ElemTable;

impl Event for WindowEvent {}

pub(super) fn emit_to_elements(table: &mut ElemTable, cursor: &mut Point, event: WindowEvent) {
    if let WindowEvent::CursorMoved {
        position: PhysicalPosition { x, y },
        ..
    } = &event
    {
        *cursor = Point(*x as _, *y as _)
    }

    tokio::runtime::Handle::current().block_on(table.emit(*cursor, &event));
}

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
    position: Point,
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
