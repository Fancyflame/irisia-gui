use crate::prim_element::{EMCreateCtx, Element, GetElement};

pub mod component;
pub mod control_flow;

pub trait VModel {
    type Storage: Model;

    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage;
    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx);
}

pub trait Model: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element));
}

pub trait VNode: VModel<Storage: GetElement> {}

impl<T> VNode for T where T: VModel<Storage: GetElement> {}

mod test {
    use crate::{self as irisia, hook::Signal};
    use irisia_macros::build2;

    use super::{component::Component, control_flow::common_vmodel::CommonVModel};

    struct Foo;

    #[derive(Default)]
    struct FooProps {
        a: Option<Signal<i32>>,
        children: Option<Signal<dyn CommonVModel>>,
        c2: Option<Signal<dyn CommonVModel>>,
    }

    impl Component for Foo {
        type Props = FooProps;
        fn create(_props: Self::Props) -> Self {
            Foo
        }
        fn render(&self) -> impl super::VModel {
            ()
        }
    }

    fn test() {
        build2! {
            Foo {
                a: 2,
                c2: build2! {
                    for i in 0..2,
                        key = i
                    {}
                },
                //children;
                if 1 + 2 == 2 {
                    /*Bar1 {
                        field1: "Aaa",
                        field2: 123,

                    }*/
                } /*else {
                    Bar2 {
                        for a in 0..10 {

                        }
                    }
                }*/
            }
        };
    }
}
