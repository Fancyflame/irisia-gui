#![allow(unused)]

use irisia_macros::Style;

fn main() {}

#[derive(Style, Clone)]
enum MyStyle {
    #[irisia(from)]
    XY { x: u32, y: u32 },

    #[irisia(from)]
    Quant {
        left: u32,
        top: u32,
        right: u32,
        bottom: u32,
    },
}

#[derive(Style, Clone)]
#[irisia(from = "t, l[, b, r] | l")]
struct StructualStyle {
    l: u32,

    #[irisia(default)]
    r: u32,

    #[irisia(default)]
    t: u32,

    #[irisia(default)]
    b: u32,
}
