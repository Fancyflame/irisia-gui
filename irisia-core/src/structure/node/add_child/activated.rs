use tokio::sync::OwnedMutexGuard;

use crate::{
    element::{Frame, RenderContent},
    structure::{
        activate::{ActivatedStructure, BareContentWrapper, Layouter, Renderable, Visit, Visitor},
        StructureBuilder,
    },
    style::StyleContainer,
    Result,
};

use super::{AddChildCache, Element, VisitItem};

pub struct AddChildActivated<El, Sb, Sty> {
    pub(super) styles: Sty,
    pub(super) children: Sb,
    pub(super) requested_size: (Option<u32>, Option<u32>),
    pub(super) el: OwnedMutexGuard<El>,
}

impl<El, Sb, Sty> ActivatedStructure for AddChildActivated<El, Sb, Sty>
where
    El: Element<Sb>,
    Sb: StructureBuilder,
    Sty: StyleContainer,
{
    type Cache = Option<AddChildCache<El, El::Props>>;

    fn element_count(&self) -> usize {
        1
    }
}

impl<El, Sb, Sty, L> Renderable<L> for AddChildActivated<El, Sb, Sty>
where
    El: Element<Sb>,
    Sb: StructureBuilder,
    Sty: StyleContainer,
    L: Layouter<El>,
{
    fn render_at(
        self,
        index: usize,
        cache: &mut Self::Cache,
        bcw: BareContentWrapper,
        layouter: &mut L,
    ) -> Result<()> {
        let AddChildActivated {
            styles,
            children,
            requested_size,
            mut el,
        } = self;
        let cache = cache.as_mut().unwrap();
        let mut bare_content = bcw.0;

        let mut content = RenderContent {
            cache_box_for_children: Some(&mut cache.cache_box),
            event_comp_index: bare_content
                .event_comp_builder
                .push(cache.init_content.element_handle.event_dispatcher().clone()),
            bare: bare_content,
        };

        let region = layouter.layout(VisitItem {
            index,
            element: &el,
            style: &styles,
            request_size: requested_size,
        })?;

        content.set_interact_region(region);
        let result = el.render(Frame {
            props: &cache.props,
            children,
            content: content.downgrade_lifetime(),
        });

        content.bare.event_comp_builder.finish();
        result
    }
}

impl<El, Sb, Sty, V> Visit<V> for AddChildActivated<El, Sb, Sty>
where
    El: Element<Sb>,
    Sb: StructureBuilder,
    Sty: StyleContainer,
    V: Visitor<El>,
{
    fn visit_at(&self, index: usize, visitor: &mut V) {
        visitor.visit(VisitItem {
            index,
            element: &self.el,
            style: &self.styles,
            request_size: self.requested_size,
        });
    }
}
