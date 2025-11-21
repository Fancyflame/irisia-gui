use irisia::hook::{Signal, watcher::Watcher};

#[tokio::main]
async fn main() {
    let person = Signal::state("Alice");
    let balance = Signal::state(100);
    let deposit = Signal::state(200);

    let account_value = Signal::memo(
        (balance.to_read(), deposit.to_read()),
        |(&balance, &deposit)| balance + deposit,
    );

    let _watch = Watcher::watch(
        (person.to_read(), account_value.clone()),
        |(&person, &value)| {
            println!("Value of {person}'s account is {value}");
        },
    );

    *balance.write() += 50;

    let (mut balance_write, mut deposit_write) = (balance.write(), deposit.write());
    *balance_write += 50;
    *deposit_write = 1000;
    drop((balance_write, deposit_write));

    test_delay_update();
}

fn test_delay_update() {
    let origin = Signal::state(0);
    let add50 = Signal::memo(origin.to_read(), |&origin| origin + 50);
    let add50_ref = add50.read();

    origin.set(100);
    dbg!(*add50_ref); // will not mutate
    drop(add50_ref);
    dbg!(*add50.read()); // mutate
}
