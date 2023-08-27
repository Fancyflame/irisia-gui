use irisia::{primitive::Pixel, Style};
use irisia::skia_safe::{Point as SkiaPoint, RRect, Rect};

#[derive(Style, Clone)]
pub enum StyleBorderRadius {
    #[style(from)]
    Radii(Pixel),

    #[style(from, from = "", impl_default)]
    Radii4 {
        #[style(option, default)]
        left_top: Pixel,

        #[style(option, default)]
        right_top: Pixel,

        #[style(option, default)]
        right_bottom: Pixel,

        #[style(option, default)]
        left_bottom: Pixel,
    },

    Oval,
}

impl StyleBorderRadius {
    pub fn oval(&mut self) {
        *self = StyleBorderRadius::Oval;
    }
}

pub(super) fn parse_border_radius(rect: &Rect, border_radius: &StyleBorderRadius) -> RRect {
    match border_radius {
        StyleBorderRadius::Radii(r) => RRect::new_rect_xy(&rect, r.to_physical(), r.to_physical()),
        StyleBorderRadius::Oval => RRect::new_oval(&rect),
        StyleBorderRadius::Radii4 {
            left_top,
            right_top,
            right_bottom,
            left_bottom,
        } => {
            fn convert(point: &Pixel) -> SkiaPoint {
                SkiaPoint::new(point.to_physical(), point.to_physical())
            }

            RRect::new_rect_radii(
                &rect,
                &[
                    convert(left_top),
                    convert(right_top),
                    convert(right_bottom),
                    convert(left_bottom),
                ],
            )
        }
    }
}
