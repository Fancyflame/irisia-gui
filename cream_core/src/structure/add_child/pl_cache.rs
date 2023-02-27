use crate::element::{proxy_layer::ProxyLayer, Element};

pub struct ProxyLayerCache<Pl, El> {
    pub pl: Pl,
    pub elem: El,
}

impl<Pl, El> Default for ProxyLayerCache<Pl, El>
where
    Pl: ProxyLayer<El>,
    El: Element,
{
    fn default() -> Self {
        ProxyLayerCache {
            pl: Default::default(),
            elem: Default::default(),
        }
    }
}
