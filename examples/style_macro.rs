#![allow(dead_code)]

use irisia::{define_style, primitive::Length, skia_safe::Color};

define_style! {
    BoxShadow:
        x Length,
        y Length,
        [
            blur Length = Length::zero(),
            spread Length = Length::zero()
        ],
        color Color = Color::BLACK;

    /// This is documentation
    #[derive(Clone)]
    Border:
        width Length,
        style &'static str = "solid",
        /,
        color Color = Color::BLACK;
}

fn main() {
    let mut sty: BoxShadow;
    sty = <_>::from((Length::px(1.0), Length::px(1.0)));
    sty.blur(Length::px(3.0)).color(Color::BLUE);

    let mut sty: Border;
    sty = <_>::from((Length::px(1.0), "dashed"));
    sty.color(Color::CYAN);
}
