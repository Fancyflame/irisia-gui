use std::marker::PhantomData;

use crate::{
    element::{render_content::BareContent, ElementMutate, InitContent},
    structure::{activate::ActivatedStructure, Element, Structure, StructureBuilder, VisitItem},
    style::StyleContainer,
};

pub use self::{activated::AddChildActivated, cache::AddChildCache};

pub mod activated;
pub mod cache;

pub struct AddChild<El, Pr, Sty, Sb, Oc> {
    _el: PhantomData<El>,
    props: Pr,
    styles: Sty,
    children: Sb,
    on_create: Oc,
}

pub fn add_child<El, Pr, Sty, Sb, Oc>(
    props: Pr,
    styles: Sty,
    children: Sb,
    on_create: Oc,
) -> AddChild<El, Pr, Sty, Sb, Oc> {
    AddChild {
        _el: PhantomData,
        props,
        styles,
        children,
        on_create,
    }
}

impl<El, Pr, Sty, Sb, Oc> Structure for AddChild<El, Pr, Sty, Sb, Oc>
where
    El: Element + ElementMutate<Pr, Sb>,
    Sb: StructureBuilder,
    Sty: StyleContainer,
    Oc: FnOnce(&InitContent<El>),
{
    type Activated = AddChildActivated<El, Pr, Sty, Sb, Oc>;

    fn activate(
        self,
        _cache: &mut <Self::Activated as ActivatedStructure>::Cache,
        _content: &BareContent,
    ) -> Self::Activated {
        AddChildActivated {
            request_size: El::compute_size(&self.props, &self.styles, &self.children),
            add_child: self,
        }
    }
}
