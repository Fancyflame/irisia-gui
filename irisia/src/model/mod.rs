use std::hash::Hash;

use crate::prim_element::{EMCreateCtx, Element, GetElement};

// pub mod component;
pub mod control_flow;

pub trait VModel {
    type Storage: Model;

    fn create(self, ctx: &EMCreateCtx) -> Self::Storage;
    fn update(self, storage: &mut Self::Storage, ctx: &EMCreateCtx);
}

pub trait Model: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element));
}

pub trait VNode: VModel<Storage: GetElement> {}

impl<T> VNode for T where T: VModel<Storage: GetElement> {}

#[cfg(disable)]
mod test {
    use crate as irisia;
    use irisia_macros::component;

    use super::VModel;
    macro_rules! test {
        ($($tt:tt)*) => {};
    }

    fn test() {
        component! {
            Foo<'a> {
                a: f32,
                s: &'a str => String,
                b: f32 => _,
                c1: _ => f32,
                c2: _ => f32,
                model children,

                Foo {
                    a: a,
                    b: b,
                    /*model slot: match c1 {
                        Some((a, b)) if 1 + 1 == 3 => children,
                        None => {},
                    },*/
                    children;
                    if a + b == 2 {
                        Bar1 {
                            field1: "Aaa",
                            field2: 123,

                        }
                    } else {
                        Bar2 {
                            for a in 0..10 {

                            }
                        }
                    }
                }
            }
        }
    }
}
