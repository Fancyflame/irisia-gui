pub struct Data {
    a: u32,
    b: String,
}

pub struct Prop<'a> {
    slice: &'a [&'static str],
    len: usize,
}

pub trait Element<Ch>: Default
where
    Ch: Node,
{
    type Props<'a>: Default;

    fn render<S, E, C>(
        &mut self,
        props: Self::Props<'_>,
        styles: S,
        event_listeners: E,
        children: Slot<Ch>,
        content: RenderContent,
    ) -> Result<()>
    where
        S: StyleContainer,
        E: EventListener,
        C: Node;
}
