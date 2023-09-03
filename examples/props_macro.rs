use irisia::{
    element::props::{PropsUpdateWith, SetStdStyles},
    style::StyleColor,
};

#[irisia::props(
    vis = "pub",
    updater = "After",
    watch(group = "some_changed", exclude = "will_change")
)]
#[allow(unused)]
struct Origin {
    #[props(watch, must_init, updated)]
    will_change: String,

    #[props(
        watch = "wont_chaaange_changed",
        default = r#""unknown".into()"#,
        updated
    )]
    wont_change: String,

    #[props(read_style(stdin))]
    abcd: Option<StyleColor>,
}

fn main() {
    let mut origin = Origin::props_create_with(
        After::default()
            .will_change("Doge")
            .wont_change("this field will not change")
            .set_std_styles(&()),
    );

    let result = origin.props_update_with(
        After::default().will_change("Cats").set_std_styles(&()),
        true,
    );

    assert!(result.will_change_changed);
    assert!(!result.wont_chaaange_changed);
    assert!(!result.some_changed);
}
