use crate::{map_rc::MapRc, renderer::Renderer};

pub trait Application: Sized {
    fn mount(slf: MapRc<Self>, renderer: &mut Renderer<Self>);
}
