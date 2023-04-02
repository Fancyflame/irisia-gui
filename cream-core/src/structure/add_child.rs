use std::{
    iter::{self, Once},
    marker::PhantomData,
    sync::Arc,
};

use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::{
    element::{Frame, PropsAsChild, RenderContent, RuntimeInit},
    event::{
        event_dispatcher::EventDispatcher,
        standard::{ElementAbondoned, EventDispatcherCreated},
        ElementEvent, EventEmitter,
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
    element_event_emitter: EventEmitter,
    children_cache: Cc,
}

struct Snapshot<El> {
    requested_size: (Option<u32>, Option<u32>),
    el: OwnedMutexGuard<El>,
}

pub struct AddChild<'a, El, Sty, Ch>
where
    El: Element,
{
    _phantom: PhantomData<El>,
    prop: <El as Element>::Props<'a>,
    style: Sty,
    event_emitter: EventEmitter,
    children: Ch,
    lock_element: Option<Snapshot<El>>,
}

pub fn add_child<El, Sty, Ch>(
    prop: <El as Element>::Props<'_>,
    style: Sty,
    event_emitter: EventEmitter,
    children: Ch,
) -> AddChild<El, Sty, Ch>
where
    El: Element,
{
    AddChild {
        _phantom: PhantomData,
        prop,
        style,
        event_emitter,
        children,
        lock_element: None,
    }
}

impl<'prop, El, Sty, Ch> RenderingNode for AddChild<'prop, El, Sty, Ch>
where
    El: Element,
    Sty: StyleContainer,
    Ch: for<'a> VisitIter<El::ChildProps<'a>>,
{
    type Cache = Option<AddChildCache<El, <Ch as RenderingNode>::Cache>>;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: RenderContent) {
        let cache = match cache {
            Some(c) => c,
            c @ None => {
                let el = Arc::new(Mutex::new(El::default()));
                let event_dispatcher = EventDispatcher::new();

                self.event_emitter
                    .emit(&EventDispatcherCreated(event_dispatcher.clone()));

                El::start_runtime(RuntimeInit {
                    _prevent_new: (),
                    app: el.clone(),
                    output_event_emitter: self.event_emitter.clone(),
                    window_event_dispatcher: content.window_event_receiver.to_recv_only(),
                    event_dispatcher: event_dispatcher.clone(),
                    close_handle: content.close_handle,
                });

                let cache = AddChildCache {
                    element: el,
                    cache_box: CacheBox::new(),
                    element_event_emitter: event_dispatcher.emitter(ElementEvent(())),
                    event_dispatcher,
                    children_cache: Default::default(),
                };

                *c = Some(cache);
                c.as_mut().unwrap()
            }
        };

        let guard = cache.element.clone().blocking_lock_owned();
        self.lock_element = Some(Snapshot {
            requested_size: guard.compute_size(),
            el: guard,
        });
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
        let cache = cache.as_mut().unwrap();
        let Snapshot {
            requested_size,
            mut el,
        } = self.lock_element.unwrap();
        let mut content = raw_content.downgrade_lifetime();

        content.elem_table_index = Some(
            content
                .elem_table_builder
                .push(cache.element_event_emitter.clone()),
        );

        let region = map(S::read_style(&self.style), requested_size)?;

        let result = el.render(Frame {
            props: self.prop,
            styles: &self.style,
            drawing_region: region,
            cache_box_for_children: &mut cache.cache_box,
            event_dispatcher: &cache.event_dispatcher,
            children: Slot {
                node: self.children,
                cache: &mut cache.children_cache,
            },
            content,
        });

        raw_content.elem_table_builder.finish();
        result
    }
}

impl<Prop, El, Sty, Ch> VisitIter<Prop> for AddChild<'_, El, Sty, Ch>
where
    El: Element + for<'a> PropsAsChild<'a, Prop>,
    Sty: StyleContainer,
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
        let locked = &self
            .lock_element
            .as_ref()
            .expect("inner error: not prepared");

        iter::once(VisitItem {
            style: S::read_style(&self.style),
            request_size: locked.requested_size,
            child_props: locked.el.props_as_child(&self.prop, &self.style),
        })
    }
}

impl<El, Cc> Drop for AddChildCache<El, Cc> {
    fn drop(&mut self) {
        self.event_dispatcher
            .emit(&ElementAbondoned, &ElementEvent(()));
    }
}
