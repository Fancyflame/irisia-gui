use irisia_core::{
    primitive::Pixel,
    skia_safe::{
        paint::Cap, Canvas, Color, Color4f, ColorSpace, Paint, PaintStyle, PathEffect,
        Point as SkiaPoint, RRect,
    },
    Style,
};
use smallvec::SmallVec;

#[derive(Style, Clone)]
#[style(
    impl_default,
    from = "width, [color,] [style | style: sliced_style(&'static [Pixel], Pixel)]"
)]
pub struct StyleBorder {
    #[style(default)]
    pub width: Pixel,

    #[style(default = "Color::BLACK", option)]
    pub color: Color,

    #[style(default = "DashStyle::Solid", option)]
    pub style: DashStyle,

    #[style(default = "Cap::Square", option)]
    pub cap: Cap,
}

#[derive(Clone)]
pub enum DashStyle {
    Owned {
        intervals: SmallVec<[Pixel; 8]>,
        phase: Pixel,
    },
    Sliced {
        intervals: &'static [Pixel],
        phase: Pixel,
    },
    Solid,
    Dotted,
}

fn sliced_style(intervals: &'static [Pixel], phase: Pixel) -> DashStyle {
    DashStyle::Sliced { intervals, phase }
}

impl StyleBorder {
    pub fn solid(&mut self) {
        self.style(DashStyle::Solid);
    }

    pub fn dotted(&mut self) {
        self.style(DashStyle::Dotted);
    }

    pub fn butt_cap(&mut self) {
        self.cap = Cap::Butt;
    }

    pub fn round_cap(&mut self) {
        self.cap = Cap::Round;
    }

    pub fn square_cap(&mut self) {
        self.cap = Cap::Square;
    }
}

// returns stroke width
pub(super) fn draw_border(canvas: &mut Canvas, mut rrect: RRect, border: &StyleBorder) -> f32 {
    let stroke_width = border.width.to_physical();

    let size_reduction = stroke_width / 2.0;
    rrect.inset(SkiaPoint::new(size_reduction, size_reduction));

    let mut paint = Paint::new(Color4f::from(border.color), &ColorSpace::new_srgb());
    paint
        .set_style(PaintStyle::Stroke)
        .set_stroke_width(stroke_width)
        .set_stroke_cap(border.cap)
        .set_anti_alias(true)
        .set_path_effect(parse_dash_style(&border.style, stroke_width));

    canvas.draw_rrect(&rrect, &paint);

    stroke_width
}

fn parse_dash_style(style: &DashStyle, stroke_width: f32) -> Option<PathEffect> {
    fn slice_dash(intervals: &[Pixel], phase: &Pixel) -> Option<PathEffect> {
        let vec: SmallVec<[f32; 12]> = intervals.iter().map(|px| px.to_physical()).collect();
        PathEffect::dash(&vec, phase.to_physical())
    }
    match style {
        DashStyle::Solid => None,
        DashStyle::Owned { intervals, phase } => slice_dash(&intervals, phase),
        DashStyle::Sliced { intervals, phase } => slice_dash(intervals, phase),
        DashStyle::Dotted => PathEffect::dash(&[stroke_width, 20.0], 0.0),
    }
}
