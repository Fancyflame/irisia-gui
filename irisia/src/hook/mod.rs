use std::rc::Rc;
pub use {listener::Listener, signal::Signal};

pub mod listener;
pub mod signal;
pub mod signal_group;
pub mod utils;
pub mod watcher;

#[macro_export]
macro_rules! coerce_hook {
    ($signal:expr) => {
        $crate::coerce_hook!($signal, _)
    };
    ($signal:expr, $TargetType:ty) => {
        (($signal).__irisia_coerce_unsized::<$TargetType>(|inner| inner as _))
    };
}

fn check_casted_rc<T, U>(a: &Rc<T>, b: &Rc<U>)
where
    T: ?Sized,
    U: ?Sized,
{
    assert_eq!(
        Rc::as_ptr(a) as *const _ as *const (),
        Rc::as_ptr(b) as *const _ as *const (),
        "the returned Rc pointer address is not equal to the input Rc"
    );
}
