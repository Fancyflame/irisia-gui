use crate::element2::CompRuntime;

use super::VModel;

pub struct Unit<Pr, Cd> {
    pub props: Pr,
    pub child_data: Cd,
}

// impl<T, Cd> VModel for Unit<T, Cd>
// where
//     T: Component,
// {
//     type Storage = CompRuntime;
//     fn create(self, ctx: &crate::el_model::EMCreateCtx) -> Self::Storage {}
// }
