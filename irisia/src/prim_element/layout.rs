use crate::primitive::rect::Rect;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SpaceConstraint {
    Exact(f32),
    Available(f32),
    MinContent,
    MaxContent,
}

impl SpaceConstraint {
    pub const fn get_numerical(&mut self) -> Option<&mut f32> {
        match self {
            Self::Exact(v) | Self::Available(v) => Some(v),
            Self::MinContent | Self::MaxContent => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct FinalLayout {
    pub region: Rect<f32>,
}
