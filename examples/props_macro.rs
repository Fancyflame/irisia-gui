use irisia::{
    element::props::{PropsUpdateWith, SetStdStyles},
    style::StyleColor,
};
use std::default::Default;

#[irisia::props(After, vis = "pub")]
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
    let mut origin = Origin::create_with(
        After::default()
            .name("Bob")
            .gender("Helicopter")
            .set_std_styles(&()),
    );

    let result = origin.update_with(After::default().name("Bob").set_std_styles(&()));
    assert!(!result.name_changed);
    assert!(result.gender_changed);
    assert!(!result.abcd_changed);
}
