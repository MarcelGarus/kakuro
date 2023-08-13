pub const LOGGING: bool = false;

#[macro_export]
macro_rules! log {
    () => {
        if $crate::log::LOGGING {
            std::println!()
        }
    };
    ($($arg:tt)*) => {
        if $crate::log::LOGGING {
            std::println!($($arg)*)
        }
    };
}
