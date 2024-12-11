use crate::{
    el_model::EMCreateCtx,
    hook::{provider_group::ProviderGroup, Consumer},
};

use super::VModel;

#[must_use]
pub struct Packer<'a, T> {
    ctx: &'a EMCreateCtx,
    storage: Storage<'a, T>,
}

enum Storage<'a, T> {
    Init(&'a mut Option<T>),
    Update(&'a mut T),
}

impl<T> Packer<'_, T> {
    pub fn apply_model<Vm>(self, vmodel: Vm)
    where
        Vm: VModel<Storage = T>,
    {
        match self.storage {
            Storage::Init(init) => *init = Some(vmodel.create(self.ctx)),
            Storage::Update(cache) => vmodel.update(cache, self.ctx),
        }
    }

    pub fn is_init(&self) -> bool {
        matches!(self.storage, Storage::Init(_))
    }
}

pub fn reactive<F, T, D>(model_maker: F, ctx: &EMCreateCtx, deps: D) -> Consumer<T>
where
    T: 'static,
    F: Fn(Packer<T>, D::Data<'_>) + 'static,
    D: ProviderGroup + 'static,
{
    let mut init = None;
    model_maker(
        Packer {
            ctx,
            storage: Storage::Init(&mut init),
        },
        D::deref_wrapper(&deps.read_many()),
    );
    let Some(init) = init else {
        panic!("`Packer` must be used");
    };

    let ctx = ctx.clone();
    Consumer::new(
        init,
        move |place, data| {
            model_maker(
                Packer {
                    ctx: &ctx,
                    storage: Storage::Update(place),
                },
                data,
            );
        },
        deps,
    )
}
