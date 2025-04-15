use crate::{
    hook::{
        reactive::{Reactive, ReactiveRef, WeakReactive},
        Signal,
    },
    model::{EleModel, Model, ModelCreateCtx, VModel},
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
        .dep2(
            move |mut storage, vmodel: &T| {
                vmodel.update(&mut *storage, &ctx);
                ReactiveRef::drop_borrow(&mut storage);
                if let Some(parent) = ctx.parent.as_ref().and_then(WeakReactive::upgrade) {
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

impl<T> EleModel for SignalModel<T>
where
    T: EleModel,
{
    fn get_element(&self) -> crate::prim_element::Element {
        self.model.as_ref().unwrap().read().get_element()
    }
}
