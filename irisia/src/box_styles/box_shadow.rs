use irisia::{skia_safe::Color, style::Pixel, Style};
use irisia_core::skia_safe::{
    canvas::SaveLayerRec, BlendMode, BlurStyle, Canvas, Color4f, ColorSpace, MaskFilter, Paint,
    RRect,
};

#[derive(Style, Clone)]
#[irisia(style(from = "radius, [spread,] [color]"))]
pub struct StyleBoxShadow {
    pub radius: Pixel,

    #[irisia(style(default))]
    pub spread: Pixel,

    #[irisia(style(default = "Color::BLACK"))]
    pub color: Color,
}

pub(super) fn draw_shadow(canvas: &mut Canvas, rrect: &RRect, style: &StyleBoxShadow) {
    let mask_filter = MaskFilter::blur(BlurStyle::Solid, style.radius.to_physical(), true);

    let mut paint = Paint::new(Color4f::from(style.color), &ColorSpace::new_srgb());
    paint
        .set_anti_alias(true)
        .set_mask_filter(mask_filter.unwrap());

    let mut clear_paint = Paint::new(Color4f::new(0.0, 0.0, 0.0, 0.0), &ColorSpace::new_srgb());
    clear_paint
        .set_anti_alias(true)
        .set_blend_mode(BlendMode::Clear);

    canvas.save_layer(&SaveLayerRec::default());
    canvas.draw_rrect(rrect, &paint);
    canvas.draw_rrect(rrect, &clear_paint);
    canvas.restore();
}
