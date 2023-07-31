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

pub struct AddChildActivated<El, Pr, Sty, Str, Oc> {
    pub(super) add_child: AddChild<El, Pr, Sty, Str, Oc>,
    pub(super) request_size: ComputeSize,
}

impl<El, Pr, Sty, Str, Oc> ActivatedStructure for AddChildActivated<El, Pr, Sty, Str, Oc>
where
    El: Element + ElementMutate<Pr, Str>,
    Str: Structure,
    Sty: StyleContainer,
{
    type Cache = Option<AddChildCache<El, ChildrenCache<El, Pr, Str>>>;

    fn element_count(&self) -> usize {
        1
    }
}

impl<El, Pr, Sty, Str, Oc, L> UpdateCache<L> for AddChildActivated<El, Pr, Sty, Str, Oc>
where
    El: Element + ElementMutate<Pr, Str>,
    Sty: StyleContainer,
    Str: Structure,
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
                c.draw_region = draw_region;
                c.element
                    .blocking_lock()
                    .update(update_arg!(&mut c.children_cache));
                c
            }
            None => {
                unchanged = false;
                let add_child_cache = AddChildCache::new(
                    &global_content,
                    draw_region,
                    |init, cache| El::create(init, update_arg!(cache)),
                    on_create,
                );
                cache.insert(add_child_cache)
            }
        };

        cache.interact_region = interact_region;

        Ok(unchanged)
    }
}

impl<El, Pr, Sty, Str, Oc, V> Visit<V> for AddChildActivated<El, Pr, Sty, Str, Oc>
where
    El: Element + ElementMutate<Pr, Str>,
    Sty: StyleContainer,
    Str: Structure,
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
