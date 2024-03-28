use std::marker::PhantomData;

use super::{StructureUpdater, VisitBy};
use crate::{
    dom::{EMCreateCtx, ElementModel},
    element::{ElementUpdate, NewState},
    style::StyleSource,
    Element,
};

pub struct Once<T> {
    pub data: T,
}

pub struct OnceUpdater<El, Pr, Sty, Su, Oc> {
    pub _phantom: PhantomData<*const El>,
    pub props: Pr,
    pub styles: Sty,
    pub slot_updater: Su,
    pub on_create: Oc,
}

impl<El> VisitBy for Once<El>
where
    El: Element,
{
    fn iter(&self) -> impl Iterator<Item = &dyn Element> {
        std::iter::once(&self.data as _)
    }

    fn visit_mut(
        &mut self,
        mut f: impl FnMut(&mut dyn Element) -> crate::Result<()>,
    ) -> crate::Result<()> {
        f(&mut self.data)
    }

    fn len(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        false
    }
}

impl<El, Pr, Sty, Su, Oc> StructureUpdater for OnceUpdater<El, Pr, Sty, Su, Oc>
where
    El: Element + ElementUpdate<Pr, Sty, Su>,
    Su: StructureUpdater,
    Sty: StyleSource,
    Oc: FnOnce(&mut El),
{
    type Target = Once<El>;

    fn create(self, ctx: &EMCreateCtx) -> Self::Target {
        let mut el = El::create(
            ElementModel::new(ctx),
            NewState {
                props: self.props,
                styles: self.styles,
                slot: self.slot_updater,
            },
        );

        (self.on_create)(&mut el);
        Once { data: el }
    }

    fn update(self, this: &mut Self::Target, _: &EMCreateCtx) {
        this.data.update(NewState {
            props: self.props,
            styles: self.styles,
            slot: self.slot_updater,
        })
    }
}
