use irisia_macros::props;
use std::default::Default;

#[irisia::props]
struct Origin {
    name: String,
    sexual_orientation: String,
    abcd: i32,
}

fn main() {
    Foo::default()
        .name("a")
        .sexual_orientation("HeliCopter")
        .abcd(1);
}
