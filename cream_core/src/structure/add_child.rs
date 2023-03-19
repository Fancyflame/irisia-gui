use std::{
    iter::{self, Once},
    marker::PhantomData,
    sync::Arc,
};

use anyhow::anyhow;
use tokio::sync::Mutex;

use crate::{
    element::RenderContent,
    event::{EventChanSetter, EventEmitter},
    style::{reader::StyleReader, StyleContainer},
    CacheBox, Result,
};

use super::{slot::Slot, Element, Node};

pub struct AddChildCache<El, Cc> {
    element: Arc<Mutex<El>>,
    cache_box: CacheBox,
    chan_setter: EventChanSetter,
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
    El: Element<Children<Ch> = Ch>,
    Sty: StyleContainer,
    Ch: Node,
{
    type Cache = Option<AddChildCache<El, <Ch as Node>::Cache>>;
    type StyleIter<'a, S> = Once<S> where Self:'a;

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader,
    {
        iter::once(S::read_style(&self.style))
    }

    fn finish_iter<'a, I>(self, cache: &mut Self::Cache, mut iter: I) -> Result<()>
    where
        I: Iterator<Item = RenderContent<'a>>,
    {
        let content: RenderContent = match iter.next() {
            Some(content) => content,
            None => {
                return Err(anyhow!(
                    "items of the render content iterator is not enough"
                ));
            }
        };

        let cache = match cache {
            Some(c) => c,
            c @ None => {
                let el = Arc::new(Mutex::new(El::create()));
                let (setter, getter) =
                    EventChanSetter::channel(content.global_event_receiver.clone());

                El::start_runtime(el.clone(), self.event_emitter, getter);

                let cache = AddChildCache {
                    element: el,
                    cache_box: CacheBox::new(),
                    chan_setter: setter,
                    children_cache: Default::default(),
                };

                *c = Some(cache);
                c.as_mut().unwrap()
            }
        };

        let mut elem = cache.element.blocking_lock();
        elem.render(
            self.prop,
            &self.style,
            &mut cache.cache_box,
            Slot {
                node: self.children,
                cache: &mut cache.children_cache,
            },
            &cache.chan_setter,
            content,
        )
    }
}
