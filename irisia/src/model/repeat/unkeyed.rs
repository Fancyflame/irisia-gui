use std::hash::Hash;

use crate::{
    el_model::EMCreateCtx,
    hook::Provider,
    model::{
        iter::{ModelMapper, VisitModel},
        ModelCreateFn,
    },
};

use super::Inserting;

pub fn repeat_unkeyed<M, V, Fv, D, Di>(get_model: Fv, dep_iter: D) -> impl ModelCreateFn<M>
where
    M: ModelMapper,
    V: VisitModel<M> + 'static,
    Fv: Fn(&Di, &EMCreateCtx) -> V + Clone + 'static,
    D: Provider + Clone + 'static,
    for<'a> &'a D::Data: IntoIterator<Item = &'a Di>,
    Di: Hash + Eq + Clone + 'static,
{
    super::repeat(
        move |data, mut updater, ctx| {
            for item in data {
                // we cannot skip this step because it marks
                // the returning element to be `used`.
                match updater.push(item.clone()) {
                    Inserting::Vacant(vac) => vac.insert(get_model(item, ctx)),
                    Inserting::Occupied(_) => {}
                }
            }
        },
        dep_iter,
    )
}
