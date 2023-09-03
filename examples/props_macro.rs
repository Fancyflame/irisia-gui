use irisia::{
    element::props::{PropsUpdateWith, SetStdStyles},
    style::StyleColor,
};

#[irisia::props(
    vis = "pub",
    updater = "After",
    watch(group = "all_unchanged", exclude = "will_change")
)]
#[allow(unused)]
struct Origin {
    #[props(watch, must_init, updated)]
    will_change: String,

    #[props(
        watch = "wont_chaaange_unchanged",
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

    let result = origin.props_update_with(After::default().will_change("Cats").set_std_styles(&()));

    assert!(!result.will_change_unchanged);
    assert!(result.wont_chaaange_unchanged);
    assert!(result.all_unchanged);
}
