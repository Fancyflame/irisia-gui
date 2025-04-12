use crate::{
    hook::{reactive::Reactive, Signal},
    model::{Model, ModelCreateCtx, VModel},
};

impl<T> VModel for Signal<T>
where
    T: VModel + ?Sized + 'static,
{
    type Storage = SignalModel<T::Storage>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        make_model(self, self.read().create(ctx), ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
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
    ctx: &ModelCreateCtx,
) -> SignalModel<T::Storage>
where
    T: VModel + ?Sized + 'static,
{
    let ctx = ctx.clone();
    let model = Reactive::builder()
        .dep(
            move |storage, vmodel: &T| {
                vmodel.update(storage, &ctx);
                if let Some(parent) = ctx.parent.upgrade() {
                    parent.push(|block_model| {
                        block_model.submit_children();
                    });
                }
            },
            vmodel.clone(),
        )
        .build(init_state);

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
