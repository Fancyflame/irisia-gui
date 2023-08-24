#[macro_export]
macro_rules! info {
    ($($tt:tt)+) => {
        $crate::log::println!("INFO", $($tt)*);
    };
}

#[macro_export]
macro_rules! warning {
    ($($tt:tt)+) => {
        $crate::log::println!("WARNING", $($tt)*);
    };
}

#[macro_export]
macro_rules! error {
    ($($tt:tt)+) => {
        $crate::log::println!("ERROR", $($tt)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! println {
    ($prefix:literal, $fmt:literal $($tt:tt)*) => {
        println!(::std::concat!("[{} {}: {}] ", $fmt), $prefix, ::std::file!(), ::std::line!() $($tt)*)
    };
}
