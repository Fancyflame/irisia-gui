use irisia::{element::props::SetStdStyles, style::StyleColor, UpdateWith};
use std::default::Default;

#[irisia::props(After)]
#[allow(unused)]
struct Origin {
    #[props(must_init, updated)]
    name: String,

    #[props(default = r#" "Walmart Bag".into() "#, updated)]
    gender: String,

    #[props(read_style(stdin))]
    abcd: Option<StyleColor>,
}

fn main() {
    Origin::create_with(
        After::default()
            .name("Bob")
            .gender("Helicopter")
            .set_std_styles(&()),
    );
}
