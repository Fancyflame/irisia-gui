use std::{any::Any, rc::Rc};

use anyhow::Result;

use crate::{
    data_driven::Watchable,
    map_rc::MapWeak,
    structure::{dynamic_children::KeyedStorage, Element},
};

#[test]
fn t() -> Result<()> {
    Rc::new_cyclic(|weak| Struct {
        children: KeyedStorage::new(|slf, key| Elem { count: key })?,
    });

    Ok(())
}

struct Elem {
    count: MapWeak<dyn Watchable<usize>>,
}

impl Element for Elem {
    type Style = ();
    type AcceptChildren = dyn Any;
    fn render(
        &mut self,
        canvas: &mut skia_safe::Canvas,
        style: &Self::Style,
        children: &[crate::map_rc::MapRc<Self::AcceptChildren>],
    ) -> anyhow::Result<()> {
        todo!()
    }
}

struct Struct {
    children: KeyedStorage<Self, usize, Elem>,
}
