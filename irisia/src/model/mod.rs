pub use control_flow::{Model, VModel, VNode};

// pub mod component;
pub mod control_flow;
pub mod optional_update;

mod test {
    use super::optional_update::DirtyPoints;

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
