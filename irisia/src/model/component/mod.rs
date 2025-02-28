pub mod dirty_flag;

pub trait ComponentArgs: Sized {
    type Model: Component<Self>;
}

pub trait Component<Args> {
    fn create(args: Args) -> Self;
    fn update(&mut self, args: Args);
}
