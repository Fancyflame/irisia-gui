pub struct ChainCallHelper<T>(T);

impl<T> ChainCallHelper<T> {
    pub fn call_func<A>(mut self, func: fn(&mut T, A), arg: A) -> Self {
        func(&mut self.0, arg);
        self
    }

    pub fn call_func_no_arg(mut self, func: fn(&mut T)) -> Self {
        func(&mut self.0);
        self
    }

    pub fn finish(self) -> T {
        self.0
    }
}

pub fn __new_chain_caller<T>(value: T) -> ChainCallHelper<T> {
    ChainCallHelper(value)
}
