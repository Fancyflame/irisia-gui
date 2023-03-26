use std::{
    iter::{self, Once},
    marker::PhantomData,
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::{
    element::{render_content::WildRenderContent, RuntimeInit},
    event::{
        event_channel::channel_map::{channel_map, getter::ELEMENT_EVENT_CHANNEL},
        standard::ElementDropped,
        EventChanSetter, EventEmitter,
    },
    primary::Region,
    style::{reader::StyleReader, StyleContainer},
    CacheBox, Result,
};

use super::{slot::Slot, Element, Node};

pub struct AddChildCache<El, Cc> {
    element: Arc<Mutex<El>>,
    cache_box: CacheBox,
    chan_setter: EventChanSetter,
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
    }
}

/*
    El: element
    Pl: proxy layer
    FProp: function that returns props
    Sty: style container
    Ls: event listeners
    Ch: children node
*/
impl<'prop, El, Sty, Ch> Node for AddChild<'prop, El, Sty, Ch>
where
    El: Element,
    Sty: StyleContainer,
    Ch: Node,
{
    type Cache = Option<AddChildCache<El, <Ch as Node>::Cache>>;
    type Iter<'a, S> = Once<S> where Self:'a;

    fn style_iter<S>(&self) -> Self::Iter<'_, S>
    where
        S: StyleReader,
    {
        iter::once(S::read_style(&self.style))
    }

    fn __finish_iter<'wrc, S, F>(
        self,
        cache: &mut Self::Cache,
        mut wild: WildRenderContent,
        map: &mut F,
    ) -> Result<()>
    where
        F: FnMut(S, Option<Region>) -> Result<Region>,
        S: StyleReader,
    {
        let mut content = wild.0.downgrade_lifetime();

        let cache = match cache {
            Some(c) => c,
            c @ None => {
                let el = Arc::new(Mutex::new(El::create()));
                let (setter, getter) = channel_map(content.global_event_receiver.clone());

                El::start_runtime(RuntimeInit {
                    _prevent_new: (),
                    app: el.clone(),
                    event_emitter: self.event_emitter,
                    channels: getter,
                    close_handle: content.close_handle,
                });

                let cache = AddChildCache {
                    element: el,
                    cache_box: CacheBox::new(),
                    element_event_emitter: tokio::runtime::Handle::current()
                        .block_on(setter.to_special_event_emitter(ELEMENT_EVENT_CHANNEL)),
                    chan_setter: setter,
                    children_cache: Default::default(),
                };

                *c = Some(cache);
                c.as_mut().unwrap()
            }
        };

        content.elem_table_index = Some(
            content
                .elem_table_builder
                .push(cache.element_event_emitter.clone()),
        );

        let mut binding = |region| map(S::read_style(&self.style), region);
        content.region_requester = Some(&mut binding);

        let result = cache.element.blocking_lock().render(
            self.prop,
            &self.style,
            &mut cache.cache_box,
            &cache.chan_setter,
            Slot {
                node: self.children,
                cache: &mut cache.children_cache,
            },
            content,
        );

        wild.0.elem_table_builder.finish();
        result
    }
}

impl<El, Cc> Drop for AddChildCache<El, Cc> {
    fn drop(&mut self) {
        let chan_setter = self.chan_setter.clone();
        tokio::spawn(async move {
            chan_setter
                .to_special_event_emitter(ELEMENT_EVENT_CHANNEL)
                .await
                .emit(&ElementDropped)
                .await;
        });
    }
}
