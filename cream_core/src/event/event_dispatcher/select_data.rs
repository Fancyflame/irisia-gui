#[macro_export]
macro_rules! select_event {
    {
        $($expr:expr => {$($tt:tt)*},)*
    } => {
        ::tokio::select! {
            $(
                __receiver = $expr => $crate::match_event! {
                    __receiver => {
                        $($tt)*
                    }
                },
            )*
        }
    };
}
