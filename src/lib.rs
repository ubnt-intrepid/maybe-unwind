/*!
A wrapper of [`catch_unwind`] that also captures the panic information.

The main purpose of this library is to provide a utility for capturing
the error information from assetion macros in custom test libraries.

[`catch_unwind`]: https://doc.rust-lang.org/stable/std/panic/fn.catch_unwind.html

# Example

```
use maybe_unwind::maybe_unwind;

maybe_unwind::set_hook();

if let Err(unwind) = maybe_unwind(|| do_something()) {
    eprintln!("payload = {:?}", unwind.payload());
    eprintln!("location = {:?}", unwind.location());
}
# fn do_something() {}
```
!*/

#![doc(html_root_url = "https://docs.rs/maybe-unwind/0.1.2")]
#![deny(missing_docs)]
#![forbid(clippy::todo, clippy::unimplemented)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(feature = "nightly", feature(backtrace))]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]

use cfg_if::cfg_if;
use lazy_static::lazy_static;
use std::{
    any::Any,
    cell::Cell,
    fmt,
    panic::{self, PanicInfo, UnwindSafe},
    ptr::NonNull,
    sync::RwLock,
};

#[cfg(feature = "nightly")]
use std::backtrace::Backtrace;

cfg_if! {
    if #[cfg(feature = "futures")] {
        mod futures;
        pub use futures::{FutureMaybeUnwindExt, MaybeUnwind};
    }
}

type PanicHook = dyn Fn(&PanicInfo) + Send + Sync + 'static;

lazy_static! {
    static ref PREV_HOOK: RwLock<Option<Box<PanicHook>>> = RwLock::new(None);
}

/// Registers the custom panic hook so that the panic information can be captured.
///
/// This function saves the current panic hook and replaces with a custom hook that
/// captures the panic information caused by closures enclosed in [`maybe_unwind`].
/// If the panic originated outside of `maybe_unwind`, the saved panic hook is
/// invoked instead.
///
/// Note that the panic hook is managed globally and replacing the hook reflects
/// all threads in the application.
///
/// [`maybe_unwind`]: ./fn.maybe_unwind.html
///
/// # Panics
///
/// This function panics if it is called from a panicking thread or the global state is poisoned.
///
/// # Data racing
///
/// This function may cause a data race if the panic hook is set from the different thread
/// at the same time. The application **must** ensure that the all dependencies that may
/// use the custom panic hook set their hooks before calling `set_hook`, and the panic hook
/// is not changed afterwards.
///
/// # Example
///
/// ```
/// use maybe_unwind::{maybe_unwind, set_hook};
///
/// set_hook();
///
/// let res = maybe_unwind(|| { panic!("oops"); });
/// assert!(res.is_err());
/// ```
#[inline]
pub fn set_hook() {
    let mut prev_hook = PREV_HOOK.write().unwrap();
    prev_hook.get_or_insert_with(|| {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(maybe_unwind_panic_hook));
        prev_hook
    });
}

/// Unregisters the custom panic hook and reset the previous hook.
#[inline]
pub fn reset_hook() {
    if let Ok(mut prev_hook) = PREV_HOOK.write() {
        if let Some(prev_hook) = prev_hook.take() {
            panic::set_hook(prev_hook);
        }
    }
}

thread_local! {
    static CAPTURED: Cell<Option<NonNull<Option<Captured>>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<Option<Captured>>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        CAPTURED.with(|captured| {
            captured.set(self.0.take());
        });
    }
}

fn maybe_unwind_panic_hook(info: &PanicInfo) {
    let captured = CAPTURED.with(|captured| captured.replace(None));
    let _reset = SetOnDrop(captured);

    match captured {
        Some(mut captured) => unsafe {
            let captured = captured.as_mut();
            captured.replace(Captured {
                location: info.location().map(|loc| Location {
                    file: loc.file().to_string(),
                    line: loc.line(),
                    column: loc.column(),
                }),
                #[cfg(feature = "nightly")]
                backtrace: Backtrace::capture(),
            });
        },
        None => fallback_hook(info),
    }
}

fn fallback_hook(info: &PanicInfo) {
    let prev_hook = PREV_HOOK.read().ok();
    let prev_hook = prev_hook.as_ref().and_then(|prev_hook| prev_hook.as_ref());
    if let Some(prev_hook) = prev_hook {
        (prev_hook)(info);
    } else {
        eprintln!("warning: the original panic hook is not available (this is a bug).");
    }
}

/// The captured information about an unwinding panic.
#[derive(Debug)]
pub struct Unwind {
    payload: Box<dyn Any + Send + 'static>,
    captured: Option<Captured>,
}

#[derive(Debug)]
struct Captured {
    location: Option<Location>,
    #[cfg(feature = "nightly")]
    backtrace: Backtrace,
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

    let res = {
        let old_info = CAPTURED.with(|tls| {
            tls.replace(Some(NonNull::from(&mut captured))) //
        });
        let _reset = SetOnDrop(old_info);
        panic::catch_unwind(f)
    };

    res.map_err(|payload| Unwind {
        payload,
        captured: captured.take(),
    })
}
