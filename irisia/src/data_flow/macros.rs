#[macro_export]
macro_rules! wire {
    ($compute:expr) => {
        $crate::wire!($compute;)
    };
    ($compute:expr; $($var:ident $(= $expr:expr)?),* $(,)?) => {
        $crate::data_flow::wire(
            |($($var,)*): &_| {$compute},
            ($(
                $crate::__clone!($var $($expr)?),
            )*)
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __clone {
    ($var:ident) => {
        $var.clone()
    };
    ($var:ident $expr:expr) => {
        $expr
    };
}
