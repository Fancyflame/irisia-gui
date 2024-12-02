use std::hash::Hash;

use crate::{
    el_model::EMCreateCtx,
    hook::{Provider, ProviderObject, State, ToProviderObject},
    model::{
        iter::{ModelMapper, VisitModel},
        ModelCreateFn,
    },
};

use super::Inserting;

pub fn repeat_keyed<M, K, V, Fk, Fv, D, Di>(
    get_key: Fk,
    get_model: Fv,
    dep_iter: D,
) -> impl ModelCreateFn<M>
where
    M: ModelMapper,
    K: Hash + Eq + Clone + 'static,
    V: VisitModel<M> + 'static,
    Fk: Fn(&Di) -> K + Clone + 'static,
    Fv: Fn(ProviderObject<Di>, &EMCreateCtx) -> V + Clone + 'static,
    D: Provider + Clone + 'static,
    for<'a> &'a D::Data: IntoIterator<Item = &'a Di>,
    Di: Clone + 'static,
{
    super::repeat(
        move |data, mut updator, ctx| {
            for item in data {
                match updator.push(get_key(item)) {
                    Inserting::Vacant(vac) => {
                        let state = State::new(item.clone());
                        vac.insert(Item {
                            model: get_model(state.to_object(), ctx),
                            iter_item: state,
                        })
                    }
                    Inserting::Occupied(occ) => occ.iter_item.set(item.clone()),
                }
            }
        },
        dep_iter,
    )
}

struct Item<T, M> {
    iter_item: State<T>,
    model: M,
}

impl<M, T, Mod> VisitModel<M> for Item<T, Mod>
where
    M: ModelMapper,
    Mod: VisitModel<M>,
{
    fn visit(&self, f: &mut dyn FnMut(M::MapRef<'_>)) {
        self.model.visit(f);
    }
    fn visit_mut(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapMut<'_>)) {
        self.model.visit_mut(f);
    }
}
