#[cfg(backtrace)]
pub(crate) use std::backtrace::Backtrace;

#[cfg(not(backtrace))]
#[derive(Debug)]
pub(crate) enum Backtrace {}

#[cfg(backtrace)]
macro_rules! capture_backtrace {
    () => {
        Some($crate::backtrace::Backtrace::capture())
    };
}

#[cfg(not(backtrace))]
macro_rules! capture_backtrace {
    () => {
        None
    };
}
