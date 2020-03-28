#[cfg(nightly)]
pub(crate) use std::backtrace::Backtrace;

#[cfg(not(nightly))]
#[derive(Debug)]
pub(crate) enum Backtrace {}

#[cfg(nightly)]
macro_rules! capture_backtrace {
    () => {
        Some($crate::backtrace::Backtrace::capture())
    };
}

#[cfg(not(nightly))]
macro_rules! capture_backtrace {
    () => {
        None
    };
}
