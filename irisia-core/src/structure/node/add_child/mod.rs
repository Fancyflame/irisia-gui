use std::marker::PhantomData;

use crate::{
    element::InitContent,
    structure::{
        activate::ActivatedStructure, BareContentWrapper, Element, Structure, StructureBuilder,
        VisitItem,
    },
    style::StyleContainer,
};

pub use self::{activated::AddChildActivated, cache::AddChildCache};

pub mod activated;
pub mod cache;

pub struct AddChild<El, Sb, Upd, Sty, Oc> {
    _el: PhantomData<El>,
    update_prop: Upd,
    styles: Sty,
    on_create: Oc,
    children: Sb,
}

pub fn add_child<El, Sb, Upd, Sty, Oc>(
    update_prop: Upd,
    styles: Sty,
    on_create: Oc,
    children: Sb,
) -> AddChild<El, Sb, Upd, Sty, Oc>
where
    El: Element<Sb>,
    Sb: StructureBuilder,
    Oc: FnOnce(&InitContent<El>),
{
    AddChild {
        _el: PhantomData,
        update_prop,
        styles,
        on_create,
        children,
    }
}

impl<El, Sb, Upd, Sty, Oc> Structure for AddChild<El, Sb, Upd, Sty, Oc>
where
    El: Element<Sb>,
    Sb: StructureBuilder,
    Upd: FnOnce(&mut El::Props),
    Sty: StyleContainer,
    Oc: FnOnce(&InitContent<El>),
{
    type Activated = AddChildActivated<El, Sb, Sty>;

    fn activate(
        self,
        cache: &mut <Self::Activated as ActivatedStructure>::Cache,
        content: &BareContentWrapper,
    ) -> Self::Activated {
        let AddChild {
            _el: _,
            update_prop,
            styles,
            on_create,
            children,
        } = self;
        let cache = cache.get_or_insert_with(|| AddChildCache::new(&content.0, on_create));
        let guard = cache.element.clone().blocking_lock_owned();
        update_prop(&mut cache.props);

        AddChildActivated {
            styles,
            children,
            requested_size: guard.compute_size(),
            el: guard,
        }
    }
}
