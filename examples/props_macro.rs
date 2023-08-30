use irisia::UpdateWith;
use std::default::Default;

#[irisia::props(After)]
#[allow(unused)]
struct Origin {
    #[props(must_init, updated)]
    name: String,

    #[props(default = r#" "Walmart Bag".into() "#, updated)]
    gender: String,

    #[props(default)]
    abcd: i32,
}

fn main() {
    Origin::create_with(After::default().name("Bob").gender("Helicopter"));
}
