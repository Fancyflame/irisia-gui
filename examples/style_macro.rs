#![allow(dead_code)]

use irisia::{define_style, primitive::Length, skia_safe::Color};

define_style! {
    BoxShadow::Full:
        x Length,
        y Length,
        blur Length = Length::zero(),
        color Color = Color::BLACK;

    BoxShadow::Color:
        color Color = Color::GREEN;

    Border:
        width Length,
        style &'static str = "solid",
        color Color = Color::BLACK;
}

fn main() {
    let mut sty: BoxShadow;
    sty = <_>::from((Length::px(1.0), Length::px(1.0)));
    sty.blur(Length::px(3.0)).color(Color::BLUE);

    let mut sty: Border;
    sty = <_>::from((Length::px(1.0),));
    sty.color(Color::CYAN);
}
