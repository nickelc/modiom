macro_rules! format_err {
    (map $($arg:tt)*) => { |_| crate::errors::Error::Message(format!($($arg)*)) };
    (ok $($arg:tt)*) => { || crate::errors::Error::Message(format!($($arg)*)) };
    ($($arg:tt)*) => { crate::errors::Error::Message(format!($($arg)*)) };
}
