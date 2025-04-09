use crate::prim_element::{EMCreateCtx, Element, GetElement};

pub mod component;
pub mod control_flow;
pub mod prim_element;

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

pub struct ModelCreateCtx {
    em_ctx: EMCreateCtx,
    parent: (),
}

mod test {
    use crate::{
        self as irisia,
        hook::Signal,
        model::prim_element::{Block, DEFAULT_LAYOUT_FN},
    };
    use irisia_macros::build2;

    use super::{component::Component, control_flow::common_vmodel::CommonVModel};

    #[derive(Default)]
    struct Foo {
        a: Option<Signal<i32>>,
        children: Option<Signal<dyn CommonVModel>>,
        c2: Option<Signal<dyn CommonVModel>>,
        b: Option<Signal<String>>,
    }

    impl Component for Foo {
        type Created = ();
        fn create(self) -> (Self::Created, impl super::VModel) {
            ((), self.children)
        }
    }

    fn test() {
        let s = Signal::state("a".to_string()).to_signal();

        build2! {
            Foo {
                a: 2,
                b:= s,
                c2: build2! {
                    for i in 0..2, key = i
                    {
                        Block {
                            layout_fn: DEFAULT_LAYOUT_FN,
                        }
                    }
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
