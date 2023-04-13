use irisia_core::{
    skia_safe::{scalar, Color},
    style::Pixel,
    Style,
};
use smallvec::SmallVec;

#[derive(Clone)]
pub enum DashStyle {
    Owned {
        intervals: SmallVec<[Pixel; 8]>,
        phase: scalar,
    },
    Sliced {
        intervals: &'static [Pixel],
        phase: Pixel,
    },
    Solid,
    Dotted,
    None,
}

#[derive(Style, Clone)]
#[irisia(from = "width[,color][,style | ,style: sliced_style(&'static [Pixel], Pixel)]")]
pub struct Border {
    pub width: Pixel,

    #[irisia(default = "Color::BLACK")]
    pub color: Color,

    #[irisia(default = "DashStyle::Solid", option)]
    pub style: DashStyle,
}

fn sliced_style(intervals: &'static [Pixel], phase: Pixel) -> DashStyle {
    DashStyle::Sliced { intervals, phase }
}

impl Border {
    pub fn none(&mut self) {
        self.style(DashStyle::None);
    }

    pub fn solid(&mut self) {
        self.style(DashStyle::Solid);
    }

    pub fn dotted(&mut self) {
        self.style(DashStyle::Dotted);
    }
}
