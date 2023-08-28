use std::default::Default;

#[irisia::props(After)]
#[allow(unused)]
struct Origin {
    name: String,
    sexual_orientation: String,
    abcd: i32,
}

fn main() {
    After::default()
        .name("a")
        .sexual_orientation("HeliCopter")
        .abcd(1);
}
