use crate::{
    hook::{reactive::Reactive, Signal},
    model::{Model, VModel},
    prim_element::EMCreateCtx,
};

impl<T> VModel for Signal<T>
where
    T: VModel + ?Sized + 'static,
{
    type Storage = SignalModel<T::Storage>;

    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        make_model(self, self.read().create(ctx), ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        if storage.vmodel_addr == self.addr() {
            return;
        }

        match storage.model.take().unwrap().into_inner() {
            Some(model) => make_model(self, model, ctx),
            None => self.create(ctx),
        };
    }
}

fn make_model<T>(
    vmodel: &Signal<T>,
    init_state: T::Storage,
    ctx: &EMCreateCtx,
) -> SignalModel<T::Storage>
where
    T: VModel + ?Sized + 'static,
{
    let model = Reactive::builder(init_state)
        .dep(
            {
                let ctx = ctx.clone();
                move |storage, vmodel: &T| {
                    vmodel.update(storage, &ctx);
                }
            },
            vmodel.clone(),
        )
        .build();

    SignalModel {
        vmodel_addr: vmodel.addr(),
        model: Some(model),
    }
}

pub struct SignalModel<T> {
    vmodel_addr: *const (),
    model: Option<Reactive<T>>,
}

impl<T> Model for SignalModel<T>
where
    T: Model,
{
    fn visit(&self, f: &mut dyn FnMut(crate::prim_element::Element)) {
        self.model.as_ref().unwrap().read().visit(f);
    }
}
