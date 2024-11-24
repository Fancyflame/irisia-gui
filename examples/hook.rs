use irisia::hook::{Consumer, Effect, Memo, ProviderObject, Ref, State, ToProviderObject};
use std::{fmt::Write, rc::Rc, time::Duration};
use tokio::{select, sync::Notify};

#[tokio::main]
async fn main() {
    let local_set = tokio::task::LocalSet::new();
    let _guard = local_set.enter();

    let text = State::new("apple".to_string());
    let factor = State::new(1);
    let number = self_increment(factor.to_object());

    let sentence = Memo::new_customized(
        String::new(),
        |mut setter, (t, n)| {
            setter.clear();
            write!(&mut *setter, "I have {} {}", *n, *t).unwrap();
        },
        (text.clone(), number.clone()),
    );

    let trailing_s = Memo::new(|count| if *count < 2 { "." } else { "s." }, number.clone());
    let final_sentence = Memo::new(|(s1, s2)| format!("{}{}", *s1, *s2), (sentence, trailing_s));

    let stop = Rc::new(Notify::new());

    let _consumer = Consumer::builder(stop.clone())
        .dep(
            |_, final_sentence| {
                println!("sentence changed: {}", &*final_sentence);
            },
            final_sentence,
        )
        .dep(
            move |stop, number| {
                if *number % 3 == 0 {
                    let mut w = factor.write();
                    *w += 1;
                    println!("increase factor to {}", *w);
                } else if *number > 15 {
                    println!("stop");
                    stop.notify_one();
                }
            },
            number,
        )
        .build();

    select! {
        _ = stop.notified() => {}
        _ = local_set => {}
    }
}

fn self_increment(factor: ProviderObject<u32>) -> Effect<u32> {
    Effect::new(
        0u32,
        |state, factor| {
            let factor = *factor;
            let handle = tokio::task::spawn_local(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                interval.tick().await;

                loop {
                    interval.tick().await;
                    let continue_ = state.update(|mut n| {
                        *n += factor;
                        true
                    });
                    if continue_ != Some(true) {
                        break;
                    }
                }
            });

            move || {
                handle.abort();
            }
        },
        factor,
    )
}
