use std::cell::Cell;

use crate::{
    dep_watch::{bitset::UsizeArray, Bitset},
    Result,
};

use super::{StructureUpdateTo, Updating, VisitBy, VisitOn};

pub struct Select<T, C, A: UsizeArray> {
    selected: bool,
    data: Option<T>,
    child: Option<C>,
    update_delay: Cell<Bitset<A>>,
}

pub struct SelectUpdater<F>(F);
pub enum SelectUpdateBranch<Tu, Cu> {
    Selected(Tu),
    NotSelected(Cu),
}

impl<T, C, A: UsizeArray> VisitBy for Select<T, C, A>
where
    T: VisitBy,
    C: VisitBy,
{
    fn visit_by<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitOn,
    {
        if self.selected {
            self.data.as_ref().unwrap().visit_by(visitor)
        } else {
            self.child.as_ref().unwrap().visit_by(visitor)
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

impl<T, C, F, Tu, Cu, A: UsizeArray> StructureUpdateTo<A> for SelectUpdater<F>
where
    T: VisitBy + 'static,
    C: VisitBy + 'static,
    F: FnOnce() -> SelectUpdateBranch<Tu, Cu>,
    Tu: StructureUpdateTo<A, Target = T>,
    Cu: StructureUpdateTo<A, Target = C>,
{
    type Target = Select<T, C, A>;
    // 1 for condition expression
    const UPDATE_POINTS: u32 = 1 + Tu::UPDATE_POINTS + Cu::UPDATE_POINTS;

    fn create(self, mut info: Updating<A>) -> Self::Target {
        match info.scoped(0, self.0) {
            SelectUpdateBranch::Selected(upd) => Select {
                selected: true,
                data: Some(upd.create(info.inherit(1, false))),
                child: None,
                update_delay: Default::default(),
            },
            SelectUpdateBranch::NotSelected(upd) => Select {
                selected: false,
                data: None,
                child: Some(upd.create(info.inherit(Tu::UPDATE_POINTS, false))),
                update_delay: Default::default(),
            },
        }
    }

    fn update(self, target: &mut Self::Target, mut info: Updating<A>) {
        if info.no_update::<Self>() {
            return;
        }

        info.step_if(0);

        match info.scoped(0, self.0) {
            SelectUpdateBranch::Selected(upd) => {
                if !target.selected {
                    info.points.union(&target.update_delay.take());
                    target.selected = true;
                }

                match &mut target.data {
                    Some(data) => upd.update(data, info.inherit(1, false)),
                    None => target.data = Some(upd.create(info.inherit(1, false))),
                }

                let mut delay_list = target.update_delay.get();
                while let Some(next) = info.points.peek() {
                    if next >= info.update_point_offset + Self::UPDATE_POINTS {
                        break;
                    }

                    info.points.step_if(next);
                    delay_list.set(next);
                }
                target.update_delay.set(delay_list);
            }

            SelectUpdateBranch::NotSelected(upd) => {
                if target.selected {
                    info.points.union(&target.update_delay.take());
                    target.selected = false;
                }

                let mut delay_list = target.update_delay.get();
                while let Some(next) = info.points.peek() {
                    if next >= info.update_point_offset + Tu::UPDATE_POINTS {
                        break;
                    }

                    info.points.step_if(next);
                    delay_list.set(next);
                }
                target.update_delay.set(delay_list);

                let new_info = info.inherit(Tu::UPDATE_POINTS + 1, false);

                match &mut target.child {
                    Some(child) => upd.update(child, new_info),
                    None => target.child = Some(upd.create(new_info)),
                }
            }
        }
    }
}
