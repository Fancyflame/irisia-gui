use std::hash::Hash;

use control_flow::{branch::Branch, execute::Execute, repeat::Repeat};
pub use control_flow::{Model, VModel, VNode};

pub mod component;
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

pub fn execute<F, R>(f: F) -> impl VModel
where
    F: FnOnce() -> R,
    R: VModel,
{
    Execute::new(f)
}

mod test {
    use super::{
        branch,
        control_flow::{branch::Branch, execute::Execute},
        execute, repeat,
        tools::DirtyPoints,
        VModel,
    };
    macro_rules! test {
        ($($tt:tt)*) => {};
    }

    fn test() {
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

    #[test]
    fn foo() {
        let expr = 10;
        let vm = execute(|| {
            if expr == 10 {
                branch(Branch::A(execute(|| repeat((0..10).map(|_| ()), |t| ()))))
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
