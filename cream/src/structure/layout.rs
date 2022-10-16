use crate::primary::Area;

use super::element::ElementHandle;

/// Given an available area and a slice of children, the layout may cut
/// the area into pieces (or leave it complete), then call `request_size`
/// on each element, provide them the size they could have. The function
/// will return the size they need (may out of the area), then the layout
/// set the area for them through optionally according to the size they
/// request.
pub trait Layout {
    fn set_area(&self, available_area: Area, children: &[ElementHandle]);
}
