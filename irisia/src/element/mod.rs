use crate::{
    dep_watch::bitset::UsizeArray,
    dom::{
        child_nodes::{ChildBox, RenderElement},
        ChildNodes,
    },
    primitive::Region,
    structure::{SlotUpdater, StructureUpdateTo},
    Result, StyleReader,
};

pub use self::props::PropsUpdateWith;
pub use crate::{application::content::GlobalContent, dom::RcElementModel};

pub mod props;

#[macro_export]
macro_rules! ElModel {
    ()=>{
        $crate::ElModel!(Self)
    };

    (_)=>{
        $crate::ElModel!(impl $crate::Element)
    };

    ($Ty: ty) => {
        $crate::element::RcElementModel<
            $Ty,
            impl $crate::StyleGroup,
            impl $crate::ChildNodes
        >
    };
}

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait Element
where
    Self: Sized + 'static,
{
    type BlankProps: Default;
    type Array: UsizeArray;

    fn render(
        &mut self,
        this: &ElModel!(),
        content: RenderElement,
        children: RenderChildren<Self::Array>,
    ) -> Result<()>;
    fn on_created(&mut self, this: &ElModel!());
    fn children(&self, slot: SlotUpdater<impl ChildNodes>) -> impl StructureUpdateTo<Self::Array>;
}

pub struct RenderChildren<'a, A: UsizeArray>(pub(crate) &'a ChildBox<A>);

impl<A: UsizeArray> RenderChildren<'_, A> {
    pub fn render(self, re: &mut RenderElement) -> Result<()> {
        self.0.render(re)
    }

    pub fn peek_styles<F, Sr>(&self, f: F)
    where
        F: FnMut(Sr),
        Sr: StyleReader,
    {
        self.0.peek_styles(f)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn layout<F, Sr>(&self, f: F) -> Result<()>
    where
        F: FnMut(Sr, usize) -> Option<Region>,
        Sr: StyleReader,
    {
        self.0.layout(f)
    }
}
