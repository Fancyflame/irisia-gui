pub use tracing;

/// Irisia Info
#[macro_export]
macro_rules! info {
    ($($tt:tt)*) => {
        $crate::tracing::info!(
            "[Irisia Info] {}",
            ::core::format_args!($($tt)*)
        )
    };
}

/// Irisia Warn
#[macro_export]
macro_rules! warn {
    ($($tt:tt)*) => {
        $crate::tracing::warn!(
            "[Irisia Warn] {}",
            ::core::format_args!($($tt)*)
        )
    };
}

/// Irisia Error
#[macro_export]
macro_rules! error {
    ($($tt:tt)*) => {
        $crate::tracing::error!(
            "[Irisia ERROR] {}",
            ::core::format_args!($($tt)*)
        )
    };
}
