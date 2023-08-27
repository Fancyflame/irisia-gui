use irisia_macros::props;
use std::default::Default;
#[props(acdc)]
struct Foo{
    name:String,
    sexual_orientation:String,
    abcd:i32
}

fn main() {
    let a = Foo{
        name: "a".into(),
        sexual_orientation: "HeliCopter".into(),
        abcd: 1,
    };
}
