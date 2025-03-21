use std::hash::Hash;

use control_flow::{branch::Branch, repeat::Repeat};
pub use control_flow::{Model, VModel, VNode};

// pub mod component;
pub mod control_flow;
pub mod tools;

pub fn branch<A, B>(b: Branch<A, B>) -> impl VModel
where
    A: VModel,
    B: VModel,
{
    b
}

pub fn repeat<I, F, K>(iter: I, key_fn: F) -> impl VModel
where
    I: Iterator,
    I::Item: VModel,
    K: Hash + Eq + Clone + 'static,
    F: Fn(&I::Item) -> K,
{
    Repeat { iter, key_fn }
}

mod test {
    use super::tools::DirtyPoints;
    macro_rules! test {
        ($($tt:tt)*) => {};
    }

    fn test(p: DirtyPoints) {
        test! {
            foo<'a> {
                a: f32,
                s: &'a str -> String,
                b1: f32 -> .,
                c1: . -> f32,
                c2: . -> f32,
            } {
                Foo {
                    a: a,
                    b: b,

                    if a + b == 2 {
                        Bar1 {

                        }
                    } else {
                        Bar2 {
                            for
                        }
                    }
                }
            }
        }
    }
}
