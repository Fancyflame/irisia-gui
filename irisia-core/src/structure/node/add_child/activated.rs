use std::marker::PhantomData;

use crate::{
    element::{ChildrenCache, ComputeSize, ElementMutate, InitContent, UpdateArguments},
    structure::{
        activate::{
            ActivatedStructure, CacheUpdateArguments, Layouter, Structure, UpdateCache, Visit,
            Visitor,
        },
        node::add_child::update_el::UpdateElementContent,
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
    Sb: Structure,
    Sty: StyleContainer,
{
    type Cache = Option<AddChildCache<El, ChildrenCache<El, Pr, Sb>>>;

    fn element_count(&self) -> usize {
        1
    }
}

impl<El, Pr, Sty, Sb, Oc, L> UpdateCache<L> for AddChildActivated<El, Pr, Sty, Sb, Oc>
where
    El: Element + ElementMutate<Pr, Sb>,
    Sty: StyleContainer,
    Sb: Structure,
    Oc: FnOnce(&InitContent<El>),
    L: Layouter<El, Pr>,
{
    fn update(self, args: CacheUpdateArguments<Self::Cache, L>) -> Result<bool> {
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

        let CacheUpdateArguments {
            offset,
            cache,
            global_content,
            layouter,
            equality_matters: mut unchanged,
        } = args;

        let draw_region = layouter.layout(VisitItem {
            index: offset,
            _el: PhantomData,
            props: &props,
            styles: &styles,
            request_size,
        })?;

        let mut interact_region = Some(draw_region);

        // TODO: use closure instead <https://github.com/rust-lang/rust/issues/97362>
        macro_rules! update_arg {
            ($cache: expr) => {
                UpdateArguments {
                    props,
                    styles,
                    children,
                    draw_region,
                    equality_matters: unchanged,
                    updater: UpdateElementContent {
                        phantom_children: PhantomData,
                        children_cache: $cache,
                        content: global_content.downgrade_lifetime(),
                        equality_matters: &mut unchanged,
                        interact_region: &mut interact_region,
                    },
                }
            };
        }

        let cache = match cache.as_mut() {
            Some(c) => {
                c.element
                    .blocking_lock()
                    .update(update_arg!(&mut c.children_cache));
                c
            }
            None => {
                unchanged = false;
                let add_child_cache = AddChildCache::new(
                    &global_content,
                    |init, cache| El::create(init, update_arg!(cache)),
                    on_create,
                );
                cache.insert(add_child_cache)
            }
        };

        cache.interact_region = interact_region;

        Ok(unchanged)
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
    Sb: Structure,
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
