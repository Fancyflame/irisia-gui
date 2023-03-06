use std::marker::PhantomData;

use cream_core::{element::Element, structure::Node, CacheBox};
use cream_macros::{build, render, style};

pub fn main() {}

#[derive(Default)]
struct MyElement {
    cache: CacheBox,
}

struct MyProps<'a> {
    name: &'a str,
    age: u8,
    is_student: bool,
}

impl Default for MyProps<'_> {
    fn default() -> Self {
        MyProps {
            name: "no name",
            age: 0,
            is_student: true,
        }
    }
}

impl Element for MyElement {
    type Props<'a> = MyProps<'a>;
    type Children<Ch: Node> = Ch;

    fn render<S, Pl, C>(
        &mut self,
        props: Self::Props<'_>,
        styles: &S,
        upstream_evl: cream_core::event::WrappedEvents,
        evl_builder: cream_core::event::EvlBuilder<Pl, Self, ()>,
        children: cream_core::structure::Slot<C>,
        content: cream_core::element::RenderContent,
    ) -> cream_core::Result<()>
    where
        S: cream_core::style::StyleContainer,
        Pl: cream_core::element::proxy_layer::ProxyLayer<Self>,
        C: cream_core::structure::Node,
        Self: Element<Children<C> = C>,
    {
        let name = "dummy";
        let value = 10;
        let optioned = Some(22);

        let ext = build! {
            @init(&evl_builder);
            MyElement{}
        };

        render! {
            @init(&evl_builder, &mut self.cache, content);

            MyElement {
                name: "John",
                age: 12,
                +style: style!{},

                for i in 0..value {
                    MyElement {
                        name: name,
                        age: i
                    }
                }

                while let Some(r) = &optioned {
                    @key *r;
                }

                if 1 > value {
                    @extend ext;
                }

                match 1 {
                    0 => {},
                    1 => {},
                    _ => MyElement {},
                }
            }
        }
    }
}
