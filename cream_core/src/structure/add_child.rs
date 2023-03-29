use std::{
    iter::{self, Once},
    marker::PhantomData,
    sync::Arc,
};

use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::{
    element::{RenderContent, RuntimeInit},
    event::{
        event_dispatcher::EventDispatcher,
        standard::{ElementAbondoned, EventDispatcherCreated},
        ElementEventKey, EventEmitter,
    },
    primary::Region,
    style::{reader::StyleReader, StyleContainer},
    CacheBox, Result,
};

use super::{slot::Slot, Element, RenderingNode};

pub struct AddChildCache<El, Cc> {
    element: Arc<Mutex<El>>,
    cache_box: CacheBox,
    event_dispatcher: EventDispatcher,
    element_event_emitter: EventEmitter,
    children_cache: Cc,
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
    lock_element: Option<((Option<u32>, Option<u32>), OwnedMutexGuard<El>)>,
}

pub fn add_child<'a, El, Sty, Ch>(
    prop: <El as Element>::Props<'a>,
    style: Sty,
    event_emitter: EventEmitter,
    children: Ch,
) -> AddChild<'a, El, Sty, Ch>
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
    Ch: RenderingNode,
{
    type Cache = Option<AddChildCache<El, <Ch as RenderingNode>::Cache>>;
    type StyleIter<'a, S> = Once<S> where Self:'a;
    type RegionIter<'a> = Once<(Option<u32>,Option<u32>)> where Self:'a;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: RenderContent) {
        let cache = match cache {
            Some(c) => c,
            c @ None => {
                let el = Arc::new(Mutex::new(El::create()));
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
                    element_event_emitter: event_dispatcher.emitter(ElementEventKey(())),
                    event_dispatcher,
                    children_cache: Default::default(),
                };

                *c = Some(cache);
                c.as_mut().unwrap()
            }
        };

        let guard = cache.element.clone().blocking_lock_owned();
        self.lock_element = Some((guard.compute_size(), guard))
    }

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader,
    {
        iter::once(S::read_style(&self.style))
    }

    fn region_iter(&self) -> Self::RegionIter<'_> {
        iter::once(self.lock_element.as_ref().unwrap().0)
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
        let (requested_region, mut el) = self.lock_element.unwrap();
        let mut content = raw_content.downgrade_lifetime();

        content.elem_table_index = Some(
            content
                .elem_table_builder
                .push(cache.element_event_emitter.clone()),
        );

        let region = map(S::read_style(&self.style), requested_region)?;

        let result = el.render(
            self.prop,
            &self.style,
            region,
            &mut cache.cache_box,
            &cache.event_dispatcher,
            Slot {
                node: self.children,
                cache: &mut cache.children_cache,
            },
            content,
        );

        raw_content.elem_table_builder.finish();
        result
    }
}

impl<El, Cc> Drop for AddChildCache<El, Cc> {
    fn drop(&mut self) {
        self.event_dispatcher
            .emit(&ElementAbondoned, &ElementEventKey(()));
    }
}
