use super::{StructureUpdater, VisitBy};
use crate::{dom::EMCreateCtx, Element};

pub struct Select<T, C> {
    selected: bool,
    data: Option<T>,
    child: Option<C>,
}

pub enum SelectState<T, C> {
    Selected(T),
    NotSelected(C),
}

impl<T, C, Item> Iterator for SelectState<T, C>
where
    T: Iterator<Item = Item>,
    C: Iterator<Item = Item>,
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Selected(s) => s.next(),
            SelectState::NotSelected(s) => s.next(),
        }
    }
}

impl<T, C> VisitBy for Select<T, C>
where
    T: VisitBy,
    C: VisitBy,
{
    fn iter(&self) -> impl Iterator<Item = &dyn Element> {
        if self.selected {
            SelectState::Selected(self.data.as_ref().unwrap().iter())
        } else {
            SelectState::NotSelected(self.child.as_ref().unwrap().iter())
        }
    }

    fn visit_mut(
        &mut self,
        f: impl FnMut(&mut dyn Element) -> crate::Result<()>,
    ) -> crate::Result<()> {
        if self.selected {
            self.data.as_mut().unwrap().visit_mut(f)
        } else {
            self.child.as_mut().unwrap().visit_mut(f)
        }
    }

    fn len(&self) -> usize {
        if self.selected {
            self.data.as_ref().unwrap().len()
        } else {
            self.child.as_ref().unwrap().len()
        }
    }
}

impl<Tu, Cu> StructureUpdater for SelectState<Tu, Cu>
where
    Tu: StructureUpdater,
    Cu: StructureUpdater,
    Tu::Target: VisitBy,
    Cu::Target: VisitBy,
{
    type Target = Select<Tu::Target, Cu::Target>;

    fn create(self, ctx: &EMCreateCtx) -> Self::Target {
        match self {
            SelectState::Selected(upd) => Select {
                selected: true,
                data: Some(upd.create(ctx)),
                child: None,
            },
            SelectState::NotSelected(upd) => Select {
                selected: false,
                data: None,
                child: Some(upd.create(ctx)),
            },
        }
    }

    fn update(self, target: &mut Self::Target, ctx: &EMCreateCtx) {
        match self {
            SelectState::Selected(upd) => {
                target.selected = true;
                match &mut target.data {
                    Some(data) => upd.update(data, ctx),
                    place @ None => *place = Some(upd.create(ctx)),
                }
            }
            SelectState::NotSelected(upd) => {
                target.selected = false;
                match &mut target.child {
                    Some(child) => upd.update(child, ctx),
                    place @ None => *place = Some(upd.create(ctx)),
                }
            }
        }
    }
}
