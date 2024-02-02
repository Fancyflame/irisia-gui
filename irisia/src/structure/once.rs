use std::marker::PhantomData;

use super::{StructureUpdateTo, Updating, VisitBy, VisitOn};
use crate::{
    dom::{DropProtection, ElementModel},
    element::{ElementCreate, RcElementModel},
    ChildNodes, Element, Result, StyleGroup,
};

pub struct Once<T, const P: u32> {
    pub data: T,
}

pub struct OnceUpdater<El, const P: u32, Fe, Fs, Su, Oc> {
    pub _phantom: PhantomData<El>,
    pub update_el: Fe,
    pub update_style: Fs,
    pub slot_updater: Su,
    pub on_create: Oc,
}

impl<El, Sty, Slt, const P: u32> VisitBy for Once<DropProtection<El, Sty, Slt>, P>
where
    El: Element,
    Sty: StyleGroup,
    Slt: ChildNodes + VisitBy,
{
    fn visit_by<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitOn,
    {
        visitor.visit_on(&self.data)
    }

    fn len(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        false
    }
}

impl<El, Sty, Slt, const P: u32, Fe, Pr, Su, Fs, Oc, const WD: usize> StructureUpdateTo<WD>
    for OnceUpdater<El, P, Fe, Fs, Su, Oc>
where
    Self: VisitBy,
    El: Element + ElementCreate<Pr>,
    Sty: StyleGroup + 'static,
    Slt: VisitBy + 'static,
    Su: StructureUpdateTo<WD, Target = Slt>,
    Fe: FnOnce(Option<&mut El>, Updating<WD>) -> Option<Pr>,
    Fs: FnOnce() -> Sty,
    Oc: FnOnce(&RcElementModel<El, Sty, Slt>),
{
    type Target = Once<DropProtection<El, Sty, Slt>, P>;
    // 1 for style update
    const UPDATE_POINTS: u32 = P + Su::UPDATE_POINTS + 1;

    fn create(self, mut info: Updating<WD>) -> Self::Target {
        let Some(props) = (self.update_el)(None, info.inherit(0, false)) else {
            panic!("element instance must be returned when the `once` node is creating");
        };

        let slot = self.slot_updater.create(info.inherit(P, false));
        let styles = info.scoped(P + Su::UPDATE_POINTS, self.update_style);
        let data = ElementModel::new(props, styles, slot);
        (self.on_create)(&data);

        Once { data }
    }

    fn update(self, target: &mut Self::Target, mut info: Updating<WD>) {
        if info.no_update::<Self>() {
            return;
        }

        if let Some(next) = info.points.peek() {
            if next < info.update_point_offset + P {
                debug_assert!((self.update_el)(
                    Some(&mut target.data.el_mut()),
                    info.inherit(0, false)
                )
                .is_none())
            }
        }

        target
            .data
            .update_slot(|slot| self.slot_updater.update(slot, info.inherit(P, false)));

        if info.step_if(P + Su::UPDATE_POINTS) {
            target
                .data
                .set_styles(info.scoped(P + Su::UPDATE_POINTS, self.update_style));
        }
    }
}
