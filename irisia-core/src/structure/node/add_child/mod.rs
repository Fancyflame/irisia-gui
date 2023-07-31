use std::marker::PhantomData;

use crate::{
    element::{ElementMutate, InitContent, SelfCache},
    structure::{Element, Structure, VisitItem},
    style::StyleContainer,
};

pub use self::{activated::AddChildActivated, cache::AddChildCache};

pub mod activated;
pub mod cache;
pub mod update_el;

pub struct AddChild<El, Pr, Sty, Str, Oc> {
    _el: PhantomData<El>,
    props: Pr,
    styles: Sty,
    children: Str,
    on_create: Oc,
}

pub fn add_child<El, Pr, Sty, Str, Oc>(
    props: Pr,
    styles: Sty,
    children: Str,
    on_create: Oc,
) -> AddChild<El, Pr, Sty, Str, Oc> {
    AddChild {
        _el: PhantomData,
        props,
        styles,
        children,
        on_create,
    }
}

impl<El, Pr, Sty, Str, Oc> Structure for AddChild<El, Pr, Sty, Str, Oc>
where
    El: Element + ElementMutate<Pr, Str>,
    Str: Structure,
    Sty: StyleContainer,
    Oc: FnOnce(&InitContent<El>),
{
    type Activated = AddChildActivated<El, Pr, Sty, Str, Oc>;

    fn activate(self, _cache: &mut SelfCache<Self>) -> Self::Activated {
        AddChildActivated {
            request_size: El::compute_size(&self.props, &self.styles, &self.children),
            add_child: self,
        }
    }
}
