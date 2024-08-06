use irisia::{
    skia_safe::{
        font_style::{Slant, Weight},
        Color,
    },
    Style,
};

pub struct FontColor(#[style(default = Color::BLACK)] pub Color);

#[derive(Style, Clone, Copy, PartialEq)]
#[style(all, derive_default)]
pub struct FontSize(#[style(default = 40.0)] pub f32);

#[derive(Style, Clone, PartialEq)]
#[style(derive_default)]
pub struct FontSlant(#[style(default = Slant::Upright)] pub Slant);

impl FontSlant {
    pub const NORMAL: Self = Self(Slant::Upright);
    pub const ITALIC: Self = Self(Slant::Italic);
    pub const OBLIQUE: Self = Self(Slant::Oblique);
}

#[derive(Style, Clone, Copy, PartialEq)]
#[style(all, derive_default)]
pub struct FontWeight(
    #[style(
        default = Weight::NORMAL,
        map(u32, Self::from_u32),
    )]
    pub Weight,
);

impl FontWeight {
    pub const EXLIGHT: Self = Self(Weight::EXTRA_LIGHT);
    pub const LIGHT: Self = Self(Weight::LIGHT);
    pub const NORMAL: Self = Self(Weight::NORMAL);
    pub const BOLD: Self = Self(Weight::BOLD);
    pub const EXBOLD: Self = Self(Weight::EXTRA_BOLD);

    fn from_u32(value: u32) -> Self {
        Self(Weight::from(
            value.try_into().expect("invalid weight value"),
        ))
    }
}
