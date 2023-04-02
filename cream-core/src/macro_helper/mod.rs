mod chain_caller;
mod for_loop;

pub use chain_caller::__new_chain_caller;
pub use for_loop::__for_loop_iter_item_as_key;

// method from `https://github.com/rust-lang/rust/issues/86935#issuecomment-1146670057`
pub type __TypeHelper<T> = T;
