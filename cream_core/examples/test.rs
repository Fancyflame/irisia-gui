use cream_core::{element::Element, structure::Node, CacheBox};
use cream_macros::{build, render, style};

pub fn main() {}

#[derive(Default)]
struct MyElement {}

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

    fn create() -> Self {
        Default::default()
    }

    fn render<S, C>(
        &mut self,
        props: Self::Props<'_>,
        styles: &S,
        cache_box: &mut CacheBox,
        children: cream_core::structure::Slot<C>,
        chan_setter: &cream_core::event::EventChanSetter,
        content: cream_core::element::RenderContent,
    ) -> cream_core::Result<()>
    where
        S: cream_core::style::StyleContainer,
        C: Node,
        Self: Element<Children<C> = C>,
    {
        let name = "dummy";
        let value = 10;
        let optioned = Some(22);

        let ext = build! {
            @init(chan_setter);
            MyElement{}
        };

        render! {
            @init(chan_setter, cache_box, content);

            MyElement {
                name: "John",
                age: 12,
                +style: style!{},

                for i in 0..value {
                    MyElement {
                        +listen "dd" => 2,
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
