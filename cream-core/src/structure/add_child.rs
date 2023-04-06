use std::{
    iter::{self, Once},
    sync::Arc,
};

use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::{
    element::{Frame, PropsAsChild, RenderContent, RuntimeInit},
    event::{
        event_dispatcher::{emitter::CreatedEventEmitter, EventDispatcher},
        standard::ElementAbondoned,
    },
    primary::Region,
    style::{reader::StyleReader, StyleContainer},
    CacheBox, Result,
};

use super::{slot::Slot, Element, RenderingNode, VisitItem, VisitIter};

pub struct AddChildCache<El, Cc> {
    element: Arc<Mutex<El>>,
    cache_box: CacheBox,
    event_dispatcher: EventDispatcher,
    children_cache: Cc,
}

pub struct AddChild<'a, El, Sty, K, Ch>(AddChildInner<'a, El, Sty, K, Ch>)
where
    El: Element;

enum AddChildInner<'a, El, Sty, K, Ch>
where
    El: Element,
{
    New {
        props: <El as Element>::Props<'a>,
        styles: Sty,
        ce_emitter: CreatedEventEmitter<'a, K>,
        children: Ch,
    },

    Preparing,

    Prepared {
        props: <El as Element>::Props<'a>,
        styles: Sty,
        children: Ch,
        requested_size: (Option<u32>, Option<u32>),
        el: OwnedMutexGuard<El>,
    },
}

pub fn add_child<'a, El, Sty, K, Ch>(
    props: <El as Element>::Props<'a>,
    styles: Sty,
    event_emitter: CreatedEventEmitter<'a, K>,
    children: Ch,
) -> AddChild<'a, El, Sty, K, Ch>
where
    El: Element,
{
    AddChild(AddChildInner::New {
        props,
        styles,
        ce_emitter: event_emitter,
        children,
    })
}

impl<'env, El, Sty, K, Ch> RenderingNode for AddChild<'env, El, Sty, K, Ch>
where
    El: Element,
    Sty: StyleContainer,
    K: Clone + Unpin + Send + 'static,
    Ch: for<'a> VisitIter<El::ChildProps<'a>>,
{
    type Cache = Option<AddChildCache<El, <Ch as RenderingNode>::Cache>>;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: RenderContent) {
        let AddChildInner::New { props: prop, styles: style, ce_emitter, children } = 
            std::mem::replace(&mut self.0, AddChildInner::Preparing)
        else {
            panic!("inner error: this node has prepared");
        };

        let cache = match cache {
            Some(c) => c,
            c @ None => {
                let el = Arc::new(Mutex::new(El::default()));
                let event_dispatcher = EventDispatcher::new();

                ce_emitter.emit(&event_dispatcher);

                El::start_runtime(RuntimeInit {
                    _prevent_new: (),
                    app: el.clone(),
                    window_event_dispatcher: content.window_event_receiver.to_recv_only(),
                    event_dispatcher: event_dispatcher.clone(),
                    close_handle: content.close_handle,
                });

                let cache = AddChildCache {
                    element: el,
                    cache_box: CacheBox::new(),
                    event_dispatcher,
                    children_cache: Default::default(),
                };

                *c = Some(cache);
                c.as_mut().unwrap()
            }
        };

        let guard = cache.element.clone().blocking_lock_owned();

        self.0 = AddChildInner::Prepared {
            props: prop,
            styles: style,
            children,
            requested_size: guard.compute_size(),
            el: guard,
        };
    }

    fn element_count(&self) -> usize {
        1
    }

    fn finish<S, F>(
        self,
        cache: &mut Self::Cache,
        mut raw_content: RenderContent,
        map: &mut F,
    ) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<Region>,
        S: StyleReader,
    {
        let AddChildInner::Prepared { props, styles, children, requested_size, mut el } = self.0
        else {
            panic!("inner error: this node is not prepared");
        };

        let cache = cache.as_mut().unwrap();
        let mut content = raw_content.downgrade_lifetime();

        content.elem_table_index = Some(
            content
                .elem_table_builder
                .push(cache.event_dispatcher.clone()),
        );

        let region = map(S::read_style(&styles), requested_size)?;

        let result = el.render(Frame {
            props,
            styles: &styles,
            drawing_region: region,
            cache_box_for_children: &mut cache.cache_box,
            event_dispatcher: &cache.event_dispatcher,
            children: Slot {
                node: children,
                cache: &mut cache.children_cache,
            },
            content,
        });

        raw_content.elem_table_builder.finish();
        result
    }
}

impl<Prop, El, Sty, K, Ch> VisitIter<Prop> for AddChild<'_, El, Sty, K, Ch>
where
    El: Element + for<'a> PropsAsChild<'a, Prop>,
    Sty: StyleContainer,
    K: Clone + Unpin + Send + 'static,
    Ch: for<'a> VisitIter<El::ChildProps<'a>>,
{
    type VisitIter<'a, S> = Once<VisitItem<S, Prop>>
    where
        S:StyleReader,
        Self:'a;

    fn visit_iter<S>(&self) -> Self::VisitIter<'_, S>
    where
        S: StyleReader,
    {
        let AddChildInner::Prepared { props, styles, requested_size, el, .. } = &self.0
        else {
            panic!("inner error: this node is not prepared");
        };

        iter::once(VisitItem {
            style: S::read_style(styles),
            request_size: *requested_size,
            child_props: el.props_as_child(props, styles),
        })
    }
}

impl<El, Cc> Drop for AddChildCache<El, Cc> {
    fn drop(&mut self) {
        self.event_dispatcher.emit_sys(ElementAbondoned);
    }
}
