use super::{GetParentPropsFn, VModel};

pub struct MapParentProps<F, V> {
    pub(super) map: F,
    pub(super) src_vmodel: V,
}

impl<F, T, R> VModel for MapParentProps<F, T>
where
    T: VModel,
    F: Fn(&T::ParentProps) -> &R,
{
    type Storage = T::Storage;
    type ParentProps = R;

    fn get_parent_props(&self, f: GetParentPropsFn<R>) {
        self.src_vmodel
            .get_parent_props(&mut |val| f((self.map)(val)));
    }

    fn create(&self, ctx: &super::ModelCreateCtx) -> Self::Storage {
        self.src_vmodel.create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &super::ModelCreateCtx) {
        self.src_vmodel.update(storage, ctx);
    }
}

impl<T> VModel for MapParentProps<(), T>
where
    T: VModel,
{
    type Storage = T::Storage;
    type ParentProps = ();

    fn get_parent_props(&self, f: GetParentPropsFn<()>) {
        self.src_vmodel.get_parent_props(&mut |_| f(&()));
    }

    fn create(&self, ctx: &super::ModelCreateCtx) -> Self::Storage {
        self.src_vmodel.create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &super::ModelCreateCtx) {
        self.src_vmodel.update(storage, ctx);
    }
}
