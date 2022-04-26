pub const LOGGING: bool = true;

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
