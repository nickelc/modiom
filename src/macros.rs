macro_rules! format_err {
    ($($arg:tt)*) => { ::errors::Error::Message(format!($($arg)*)) }
}
