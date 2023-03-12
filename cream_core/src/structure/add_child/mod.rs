use std::{
    iter::{self, Once},
    marker::PhantomData,
};

use anyhow::anyhow;

use crate::{
    element::{proxy_layer::ProxyLayer, RcHandle, RenderContent},
    event::event_state::{build::EventListenerBuilder, proxy::EvlProxyBuilder, EventResolve},
    style::{reader::StyleReader, StyleContainer},
    Result,
};

use self::pl_cache::ProxyLayerCache;

use super::{slot::Slot, Element, Node};

pub(crate) mod pl_cache;

pub struct AddChildCache<Pl, El, Cc> {
    pl_cache: RcHandle<ProxyLayerCache<Pl, El>>,
    children_cache: Cc,
}

pub struct AddChild<'a, El, Pl, Sty, Ch, El2, Pl2, L>
where
    El: Element,
{
    _phantom: PhantomData<(El, Pl)>,
    prop: <El as Element>::Props<'a>,
    style: Sty,
    listeners: EventListenerBuilder<Pl2, El2, L>,
    children: Ch,
}

pub fn add_child<'a, El, Pl, Sty, Ch, El2, Pl2, L>(
    prop: <El as Element>::Props<'a>,
    style: Sty,
    listeners: EventListenerBuilder<Pl2, El2, L>,
    children: Ch,
) -> AddChild<'a, El, Pl, Sty, Ch, El2, Pl2, L>
where
    El: Element,
{
    AddChild {
        _phantom: PhantomData,
        prop,
        style,
        listeners,
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
impl<'prop, El, Pl, Sty, Ch, El2, Pl2, L> Node for AddChild<'prop, El, Pl, Sty, Ch, El2, Pl2, L>
where
    El: Element<Children<Ch> = Ch> + 'static,
    Pl: ProxyLayer<El>,
    Sty: StyleContainer,
    Ch: Node,
    El2: Element,
    Pl2: ProxyLayer<El2>,
    L: EventResolve<Pl2, El2>,
{
    type Cache = Option<AddChildCache<Pl, El, <Ch as Node>::Cache>>;
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
        let content = match iter.next() {
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
                let cache = AddChildCache {
                    pl_cache: Default::default(),
                    children_cache: Default::default(),
                };
                *c = Some(cache);
                c.as_mut().unwrap()
            }
        };

        let mut pl_cache_refmut = cache.pl_cache.borrow_mut();
        let pl_cache = &mut *pl_cache_refmut;

        pl_cache.pl.proxy(
            &mut pl_cache.elem,
            self.prop,
            &self.style,
            EvlProxyBuilder::from_builder(self.listeners),
            EventListenerBuilder::new(&cache.pl_cache),
            Slot {
                node: self.children,
                cache: &mut cache.children_cache,
            },
            content,
        )
    }
}
