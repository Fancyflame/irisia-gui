use crate::primitive::{Point, Size, length::LengthStandard, rect::Rect};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SpaceConstraint {
    Exact(f32),
    Available(f32),
    MinContent,
    MaxContent,
}

impl SpaceConstraint {
    pub const fn get_numerical_mut(&mut self) -> Option<&mut f32> {
        match self {
            Self::Exact(v) | Self::Available(v) => Some(v),
            Self::MinContent | Self::MaxContent => None,
        }
    }

    pub const fn get_numerical(&self) -> Option<f32> {
        match self {
            Self::Exact(v) | Self::Available(v) => Some(*v),
            Self::MinContent | Self::MaxContent => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct LayoutInput {
    pub constraint: Size<SpaceConstraint>,
    pub length_standard: Size<LengthStandard>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FinalLayout {
    pub size: Size<f32>,
    pub location: Point<f32>,
}

impl FinalLayout {
    pub const HIDDEN: Self = Self {
        size: Size::all(0.0),
        location: Point::all(0.0),
    };

    pub const fn as_rect(&self) -> Rect<f32> {
        Rect::from_location_size(self.location, self.size)
    }

    pub const fn is_hidden(&self) -> bool {
        self.size.is_empty()
    }
}
