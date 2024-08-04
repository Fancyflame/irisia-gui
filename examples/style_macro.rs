#![allow(dead_code)]

use irisia::{skia_safe::Color, style, Style};

#[derive(Style, Clone)]
#[style("size [color [padding]] [rounded]")]
struct StructStyle {
    size: f32,

    #[style(default)]
    padding: f32,

    #[style(default = Color::RED)]
    color: Color,

    #[style(default)]
    rounded: bool,
}

#[derive(Style, Clone)]
enum EnumStyle {
    #[style(all)]
    FourCorners {
        top_left: f32,
        top_right: f32,
        bottom_left: f32,
        bottom_right: f32,
    },

    #[style("x y [absolute]")]
    XY {
        x: f32,
        y: f32,

        #[style(default)]
        absolute: bool,
    },
}

fn main() {
    // all of them are valid
    let _ = style! {
        StructStyle: 20.0, Color::RED, 5.0, true;
        StructStyle: 20.0, false;
        StructStyle: 20.0, Color::BLUE;
        EnumStyle: 1.0, 2.0, 3.0, 4.0;
        EnumStyle: 1.0, 2.0;
        EnumStyle: 1.0, 2.0, true;
    };
}
