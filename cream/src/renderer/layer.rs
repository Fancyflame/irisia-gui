use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use anyhow::Result;
use getset::{CopyGetters, Getters, MutGetters};
use smallvec::SmallVec;

use crate::{
    event::{callback::ClosureCall, event_target::EventTarget, Event},
    map_rc::{MapRc, MapWeak},
    primary::Area,
    structure::Element,
};

/*pub(crate) type LayerHandle = MapRc<RefCell<dyn AnyLayer>>;
pub(crate) type LayerHandleWeak = MapWeak<RefCell<dyn AnyLayer>>;*/

#[derive(Getters, CopyGetters, MutGetters)]
pub struct Layer<E>
where
    E: Element,
{
    #[getset(get = "pub", get_mut)]
    ele: E,

    style: E::Style,

    #[getset(get = "pub")]
    graphic: Vec<u8>,

    #[getset(get_copy = "pub")]
    area: Area,

    #[getset(get = "pub", get_mut = "pub")]
    event_target: EventTarget,
}

//impl<El> Layer<El> where El: Element {}
