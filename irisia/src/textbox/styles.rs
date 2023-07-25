use irisia_core::{
    primitive::Pixel,
    skia_safe::font_style::{Slant, Weight},
    Style,
};

#[derive(Style, Clone, Copy, PartialEq)]
#[style(from, impl_default)]
pub struct StyleFontSize(#[style(default = "Pixel(40.0)")] pub Pixel);

#[derive(Style, Clone, PartialEq)]
#[style(from = "", impl_default)]
pub struct StyleFontSlant(#[style(default = "Slant::Upright")] pub Slant);

impl StyleFontSlant {
    pub fn normal(&mut self) {
        self.0 = Slant::Upright
    }

    pub fn italic(&mut self) {
        self.0 = Slant::Italic
    }

    pub fn oblique(&mut self) {
        self.0 = Slant::Oblique
    }
}

#[derive(Style, Clone, Copy, PartialEq)]
#[style(from = "[0]", impl_default)]
pub struct StyleFontWeight(#[style(default = "Weight::NORMAL")] pub Weight);

impl StyleFontWeight {
    pub fn exlight(&mut self) {
        self.0 = Weight::EXTRA_LIGHT;
    }

    pub fn light(&mut self) {
        self.0 = Weight::LIGHT;
    }

    pub fn normal(&mut self) {
        self.0 = Weight::NORMAL;
    }

    pub fn bold(&mut self) {
        self.0 = Weight::BOLD;
    }

    pub fn exbold(&mut self) {
        self.0 = Weight::EXTRA_BOLD;
    }
}

impl From<u32> for StyleFontWeight {
    fn from(value: u32) -> Self {
        StyleFontWeight(Weight::from(value as i32))
    }
}
