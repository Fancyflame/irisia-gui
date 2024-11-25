use crate::el_model::ElementModel;
pub(crate) use sealed::Sealed;

pub type ModelRefIter<'i, 'r, M> = &'i mut dyn Iterator<Item = <M as ModelMapper>::MapRef<'r>>;
pub type ModelMutIter<'i, 'r, M> = &'i mut dyn Iterator<Item = <M as ModelMapper>::MapMut<'r>>;

mod sealed {
    pub trait Sealed {}
}

pub trait DynIterModel<M: ModelMapper>: Sealed {
    fn __receive_iter<'a>(&'a self, f: &mut dyn FnMut(ModelRefIter<'_, 'a, M>));
    fn __receive_iter_mut<'a>(&'a mut self, f: &mut dyn FnMut(ModelMutIter<'_, 'a, M>));
}

pub trait IterModel<M: ModelMapper>: DynIterModel<M> {
    fn iter<'a, F, R>(&'a self, f: F) -> R
    where
        F: FnOnce(ModelRefIter<'_, 'a, M>) -> R,
    {
        let mut f = Some(f);
        let mut result = None;
        self.__receive_iter(&mut |iter| {
            result = Some(f.take().unwrap()(iter));
        });
        result.unwrap()
    }

    fn iter_mut<'a, F, R>(&'a mut self, f: F) -> R
    where
        F: FnOnce(ModelMutIter<'_, 'a, M>) -> R,
    {
        let mut f = Some(f);
        let mut result = None;
        self.__receive_iter_mut(&mut |iter| {
            result = Some(f.take().unwrap()(iter));
        });
        result.unwrap()
    }
}

impl<M, T> IterModel<M> for T
where
    M: ModelMapper,
    T: DynIterModel<M> + ?Sized,
{
}

pub trait ModelMapper: 'static {
    type MapRef<'a>;
    type MapMut<'a>;
}

pub trait ModelMapperImplements<El, Cp>: ModelMapper {
    fn map_ref(model: &ElementModel<El, Cp>) -> Self::MapRef<'_>;
    fn map_mut(model: &mut ElementModel<El, Cp>) -> Self::MapMut<'_>;
}
