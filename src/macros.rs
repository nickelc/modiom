macro_rules! format_err {
    ($($arg:tt)*) => { crate::errors::Error::Message(format!($($arg)*)) }
}
