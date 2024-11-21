use super::{memo::Memo, state::State};
use std::fmt::Write;

#[test]
fn test() {
    let text = State::new("apple".to_string());
    let number = State::new(1);

    let sentence = Memo::new_customized(
        String::new(),
        |mut setter, (t, n)| {
            setter.clear();
            write!(&mut *setter, "I have {} {}", *n, *t).unwrap();
        },
        (text.clone(), number.clone()),
    );

    let trailing_s = Memo::new(|count| if *count < 2 { "." } else { "s." }, number.clone());
    let final_sentence = Memo::new(
        |(s1, s2)| {
            println!("generated a new sentence");
            format!("{}{}", *s1, *s2)
        },
        (sentence, trailing_s),
    );

    assert_eq!(*final_sentence.read(), "I have 1 apple.");
    number.set(50);
    assert_eq!(*final_sentence.read(), "I have 50 apples.");
}
