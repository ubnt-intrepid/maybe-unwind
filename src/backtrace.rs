#[cfg(nightly)]
pub use std::backtrace::Backtrace;

#[cfg(not(nightly))]
pub use fallback::Backtrace;

#[cfg(not(nightly))]
mod fallback {
    use std::fmt;

    /// A placeholder type that emulates `std::backtrace::Backtrace` in the stable channel.
    ///
    /// Currently, this type does not support for capturing the stack backtrace.
    #[derive(Debug)]
    pub struct Backtrace(());

    impl Backtrace {
        pub(crate) fn capture() -> Self {
            Self(())
        }
    }

    impl fmt::Display for Backtrace {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("unsupported backtrace")
        }
    }
}
