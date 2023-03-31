#![allow(unused)]

use cream_core::style::AddStyle;
use cream_core::style::NoStyle;
use cream_core::style::Pixel;
use cream_core::style::Style;
use cream_core::style::StyleContainer;
use cream_macros::style;

fn main() {
    type ExtStyle = cream_core::style::Chain<
        cream_core::style::Chain<NoStyle, AddStyle<StyleStyle2>>,
        AddStyle<StyleStyle1>,
    >;

    let ext_style = style! {
        style2: false;
        style1: "looo";
    };

    let my_str = Some("hello world");
    style! {
        style1: 10, 10;

        if 1 + 1 == 2 {
            style2: true;
        }

        match my_str {
            None => {
                style1: 10, 20;
                @extend ext_style;
            }

            Some("ddd")=>{},

            Some(other) => {
                style1: &other[..5];
                pixel_test: 2px, 6.2px, .scale 10.2, .empty;
            }
        }
    };
}

#[derive(Clone)]
struct StyleStyle1;

impl Style for StyleStyle1 {}

impl From<(u32, u32)> for StyleStyle1 {
    fn from(_: (u32, u32)) -> Self {
        Self
    }
}

impl From<(&'static str,)> for StyleStyle1 {
    fn from(_: (&'static str,)) -> Self {
        Self
    }
}

#[derive(Clone)]
struct StyleStyle2;

impl Style for StyleStyle2 {}

impl From<(bool,)> for StyleStyle2 {
    fn from(_: (bool,)) -> Self {
        Self
    }
}

#[derive(Clone)]
struct StylePixelTest;

impl Style for StylePixelTest {}

impl From<(Pixel,)> for StylePixelTest {
    fn from(_value: (Pixel,)) -> Self {
        Self
    }
}

impl From<(Pixel, Pixel)> for StylePixelTest {
    fn from(_value: (Pixel, Pixel)) -> Self {
        Self
    }
}

impl StylePixelTest {
    fn empty(&mut self) {}
    fn scale(&mut self, _mul: f32) {}
}
