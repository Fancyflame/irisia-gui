use cream_backend::skia_safe::font_style::{Slant, Weight};
use cream_core::{style::Pixel, Style};

#[derive(Style, Clone, Copy)]
#[cream(from, impl_default)]
pub struct StyleFontSize(#[cream(default = "Pixel(40.0)")] pub Pixel);

#[derive(Style, Clone)]
#[cream(from, impl_default)]
pub struct StyleFontSlant(#[cream(default = "Slant::Upright")] pub Slant);

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

#[derive(Style, Clone, Copy)]
#[cream(from, impl_default)]
pub struct StyleFontWeight(#[cream(default = "Weight::NORMAL")] pub Weight);

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
