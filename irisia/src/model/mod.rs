use std::hash::Hash;

use control_flow::{branch::Branch, execute::Execute, repeat::Repeat};
pub use control_flow::{Model, VModel, VNode};

pub mod component;
pub mod control_flow;
pub mod tools;

mod test {
    use irisia_macros::component;

    use crate::model::control_flow::branch::Branch;

    use super::{
        control_flow::{branch, execute, repeat},
        tools::DirtyPoints,
        VModel,
    };
    macro_rules! test {
        ($($tt:tt)*) => {};
    }

    fn test() {
        use crate as irisia;
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
                            /*for a in 0..10 {

                            }*/
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn foo() {
        let expr = 10;
        let vm = execute(|| {
            if expr == 10 {
                branch(Branch::A(execute(|| repeat((0..10).map(|idx| (idx, ()))))))
            } else {
                branch(Branch::B(execute(|| ())))
            }
        });
        assert_eq!(get_exec_point(&vm), 3);
    }

    fn get_exec_point<T: VModel>(_: &T) -> usize {
        T::EXECUTE_POINTS
    }
}
