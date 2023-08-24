use irisia_core::{primitive::Pixel, Style};

#[derive(Style, Clone, Copy)]
#[style(from, from = "", impl_default)]
pub struct StyleMargin {
    #[style(option, default)]
    pub left: Pixel,

    #[style(option, default)]
    pub top: Pixel,

    #[style(option, default)]
    pub right: Pixel,

    #[style(option, default)]
    pub bottom: Pixel,
}

impl From<(Pixel,)> for StyleMargin {
    fn from((px,): (Pixel,)) -> Self {
        Self {
            left: px,
            top: px,
            right: px,
            bottom: px,
        }
    }
}

impl From<(Pixel, Pixel)> for StyleMargin {
    fn from((x, y): (Pixel, Pixel)) -> Self {
        Self {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }
}
