use crate::{
    application::IncomingPointerEvent, el_model::ElementModel, element::Render, primitive::Region,
    ElementInterfaces, Result,
};

use super::{ModelMapper, ModelMapperImplements};

pub trait ModelBasic: 'static {
    fn dyn_render(&mut self, args: Render) -> Result<()>;
    fn dyn_set_draw_region(&mut self, region: Option<Region>);
    fn dyn_on_pointer_event(&mut self, ipe: &IncomingPointerEvent) -> bool;
    fn dyn_layout(&mut self, region: Option<Region>);
}

impl<El, Cp, Slt> ModelBasic for ElementModel<El, Cp, Slt>
where
    El: ElementInterfaces,
    Cp: 'static,
    Slt: 'static,
{
    fn dyn_render(&mut self, args: Render) -> Result<()> {
        self.render(args)
    }
    fn dyn_set_draw_region(&mut self, region: Option<Region>) {
        self.set_draw_region(region);
    }
    fn dyn_on_pointer_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.on_pointer_event(ipe)
    }
    fn dyn_layout(&mut self, region: Option<Region>) {
        self.set_draw_region(region);
    }
}

impl ModelMapper for () {
    type MapRef<'a> = &'a dyn ModelBasic;
    type MapMut<'a> = &'a mut dyn ModelBasic;
}

impl<El, Cp, Slt> ModelMapperImplements<El, Cp, Slt> for ()
where
    El: ElementInterfaces,
    Cp: 'static,
    Slt: 'static,
{
    fn map_ref(model: &ElementModel<El, Cp, Slt>) -> Self::MapRef<'_> {
        model
    }

    fn map_mut(model: &mut ElementModel<El, Cp, Slt>) -> Self::MapMut<'_> {
        model
    }
}

impl<'a> AsRef<dyn ModelBasic + 'a> for &(dyn ModelBasic + 'a) {
    fn as_ref(&self) -> &(dyn ModelBasic + 'a) {
        *self
    }
}

impl<'a> AsMut<dyn ModelBasic + 'a> for &mut (dyn ModelBasic + 'a) {
    fn as_mut(&mut self) -> &mut (dyn ModelBasic + 'a) {
        *self
    }
}
