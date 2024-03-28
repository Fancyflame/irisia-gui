use std::time::Duration;

use irisia_backend::skia_safe::Canvas;

use crate::{
    application::{event_comp::IncomingPointerEvent, redraw_scheduler::StandaloneRender},
    dom::{
        layer::{LayerCompositer, LayerRebuilder},
        ElementModel, NewLayerBase,
    },
    primitive::Region,
    structure::StructureUpdater,
    style::StyleSource,
    Result,
};

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait Element: 'static {
    type BlankProps: Default
    where
        Self: Sized;

    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;
    fn set_draw_region(&mut self, dr: Region);
    fn on_pointer_event(&mut self, ipe: &IncomingPointerEvent) -> bool;
    fn get_style(&self) -> &dyn StyleSource;
}

pub trait ElementUpdate<Pr, Sty, Slt>
where
    Self: Sized,
    Sty: StyleSource,
    Slt: StructureUpdater,
{
    fn create(em: ElementModel, new_state: NewState<Pr, Sty, Slt>) -> Self;
    fn update(&mut self, new_state: NewState<Pr, Sty, Slt>);
}

pub trait SelfRender: 'static {
    fn self_render<'a, T>(&self, base: T, interval: Duration) -> Result<()>
    where
        T: NewLayerBase<'a>;
}

pub struct NewState<Pr, Sty, Slt> {
    pub props: Pr,
    pub styles: Sty,
    pub slot: Slt,
}

impl<El> StandaloneRender for El
where
    El: SelfRender,
{
    fn standalone_render(&self, canvas: &Canvas, interval: Duration) -> Result<()> {
        struct Wrapper<'a>(&'a Canvas);
        impl<'a> NewLayerBase<'a> for Wrapper<'a> {
            fn create_from(self, em: &'a mut ElementModel) -> Result<LayerRebuilder<'a>> {
                Ok(LayerCompositer::rebuild(
                    em.get_or_init_indep_layer(),
                    self.0,
                ))
            }
        }

        self.self_render(Wrapper(canvas), interval)
    }
}
