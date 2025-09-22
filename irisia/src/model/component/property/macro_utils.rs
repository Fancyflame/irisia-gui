use crate::{Property, model::component::property::PropExtend};

pub type EmptyOf<T> = <T as Property>::EmptyProp;

pub type PropExtendResult<Src, Updator, UpdateResult> =
    <Src as PropExtend<Updator>>::Output<UpdateResult>;

pub const fn get_empty<P: Property>() -> EmptyOf<P> {
    P::__IRISIA_EMPTY_PROP
}
