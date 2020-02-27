use crate::tls::Context;
use std::{
    any::Any,
    fmt,
    panic::{self, UnwindSafe},
};

#[cfg(feature = "nightly")]
use std::backtrace::Backtrace;

/// Invokes a closure, capturing the cause of an unwinding panic if one occurs.
///
/// In addition, this function also captures the panic information if the custom
/// panic hook is set. If the panic hook is not set, only the cause of unwinding
/// panic captured by `catch_unwind` is returned.
pub fn maybe_unwind<F, R>(f: F) -> Result<R, Unwind>
where
    F: FnOnce() -> R + UnwindSafe,
{
    let mut captured: Option<Captured> = None;

    let res = Context {
        captured: &mut captured,
    }
    .scope(|| panic::catch_unwind(f));

    res.map_err(|payload| Unwind {
        payload,
        captured: captured.take(),
    })
}

/// The captured information about an unwinding panic.
#[derive(Debug)]
pub struct Unwind {
    payload: Box<dyn Any + Send + 'static>,
    captured: Option<Captured>,
}

#[derive(Debug)]
pub(crate) struct Captured {
    pub(crate) location: Option<Location>,
    #[cfg(feature = "nightly")]
    pub(crate) backtrace: Backtrace,
}

impl Unwind {
    /// Return the payload associated with the captured panic.
    #[inline]
    pub fn payload(&self) -> &(dyn Any + Send + 'static) {
        &*self.payload
    }

    /// Return the string representation of the panic payload.
    #[inline]
    pub fn payload_str(&self) -> &str {
        let payload = self.payload();
        (payload.downcast_ref::<&str>().copied())
            .or_else(|| payload.downcast_ref::<String>().map(|s| s.as_str()))
            .unwrap_or_else(|| "Box<dyn Any>")
    }

    /// Convert itself into a trait object of the panic payload.
    #[inline]
    pub fn into_payload(self) -> Box<dyn Any + Send + 'static> {
        self.payload
    }

    /// Return the information about the location from which the panic originated.
    #[inline]
    pub fn location(&self) -> Option<&Location> {
        self.captured.as_ref()?.location.as_ref()
    }

    /// Return the captured backtrace for the panic.
    #[cfg(feature = "nightly")]
    #[cfg_attr(feature = "nightly", doc(cfg(feature = "nightly")))]
    #[inline]
    pub fn backtrace(&self) -> Option<&Backtrace> {
        Some(&self.captured.as_ref()?.backtrace)
    }
}

impl fmt::Display for Unwind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.location() {
            Some(location) => write!(f, "[{}] {}", location, self.payload_str()),
            None => f.write_str(self.payload_str()),
        }
    }
}

/// The information about the location of an unwinding panic.
#[derive(Debug)]
pub struct Location {
    file: String,
    line: u32,
    column: u32,
}

impl Location {
    #[inline]
    pub(crate) fn from_std(loc: &panic::Location<'_>) -> Self {
        Self {
            file: loc.file().to_string(),
            line: loc.line(),
            column: loc.column(),
        }
    }

    /// Return the name of the source file from which the panic originated.
    #[inline]
    pub fn file(&self) -> &str {
        self.file.as_str()
    }

    /// Return the line number from which the panic originated.
    #[inline]
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Return the column from which the panic originated.
    #[inline]
    pub fn column(&self) -> u32 {
        self.column
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}
