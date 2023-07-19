use std::marker::PhantomData;

use crate::{
    element::{ComputeSize, ElementMutate, InitContent, UpdateArguments},
    structure::{
        activate::{
            ActivateUpdateArguments, ActivatedStructure, Layouter, Renderable, Visit, Visitor,
        },
        StructureBuilder,
    },
    style::StyleContainer,
    Result,
};

use super::{AddChild, AddChildCache, Element, VisitItem};

pub struct AddChildActivated<El, Pr, Sty, Sb, Oc> {
    pub(super) add_child: AddChild<El, Pr, Sty, Sb, Oc>,
    pub(super) request_size: ComputeSize,
}

impl<El, Pr, Sty, Sb, Oc> ActivatedStructure for AddChildActivated<El, Pr, Sty, Sb, Oc>
where
    El: Element + ElementMutate<Pr, Sb>,
    Sb: StructureBuilder,
    Sty: StyleContainer,
{
    type Cache = Option<AddChildCache<El>>;

    fn element_count(&self) -> usize {
        1
    }
}

impl<El, Pr, Sty, Sb, Oc, L> Renderable<L> for AddChildActivated<El, Pr, Sty, Sb, Oc>
where
    El: Element + ElementMutate<Pr, Sb>,
    Sty: StyleContainer,
    Sb: StructureBuilder,
    Oc: FnOnce(&InitContent<El>),
    L: Layouter<El, Pr>,
{
    fn update(self, args: ActivateUpdateArguments<Self::Cache, L>) -> Result<bool> {
        let AddChildActivated {
            add_child:
                AddChild {
                    _el: _,
                    props,
                    styles,
                    children,
                    on_create,
                },
            request_size,
        } = self;

        let ActivateUpdateArguments {
            offset,
            cache,
            bare_content,
            layouter,
            equality_matters,
        } = args;

        let draw_region = layouter.layout(VisitItem {
            index: offset,
            _el: PhantomData,
            props: &props,
            styles: &styles,
            request_size,
        })?;

        let update_args = UpdateArguments {
            props,
            styles,
            children,
            draw_region,
            equality_matters,
        };

        let the_same = match cache.as_mut() {
            Some(c) => c.element.blocking_lock().update(update_args),
            None => {
                let add_child_cache = AddChildCache::new(
                    &bare_content,
                    |init| El::create(init, update_args),
                    on_create,
                );
                *cache = Some(add_child_cache);
                false
            }
        };

        Ok(the_same)
        /*let mut independent_layer = cache.independent_layer.as_ref().map(|x| x.borrow_mut());
        let rebuilder = match &mut independent_layer {
            Some(mut il) => rebuilder.new_layer(&mut il)?,
            None => rebuilder,
        };

        let mut content = RenderContent {
            cache_box_for_children: Some(&mut cache.cache_box),
            event_comp_index: bare_content
                .event_comp_builder
                .push(cache.init_content.element_handle.event_dispatcher().clone()),
            bare: bare_content,
            layer_rebuilder: rebuilder,
        };

        let region = layouter.layout(VisitItem {
            index: offset,
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
        result*/
    }
}

impl<El, Pr, Sty, Sb, Oc, V> Visit<V> for AddChildActivated<El, Pr, Sty, Sb, Oc>
where
    El: Element + ElementMutate<Pr, Sb>,
    Sty: StyleContainer,
    Sb: StructureBuilder,
    V: Visitor<El, Pr>,
{
    fn visit_at(&self, offset: usize, visitor: &mut V) {
        visitor.visit(VisitItem {
            index: offset,
            _el: PhantomData,
            props: &self.add_child.props,
            styles: &self.add_child.styles,
            request_size: self.request_size,
        });
    }
}
