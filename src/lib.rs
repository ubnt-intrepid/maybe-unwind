/*!
A variant of [`catch_unwind`] that also captures the panic information.

The main purpose of this library is to provide a utility for capturing
the error information from assetion macros in custom test libraries.

[`catch_unwind`]: https://doc.rust-lang.org/stable/std/panic/fn.catch_unwind.html

# Example

```no_run
use maybe_unwind::maybe_unwind;

maybe_unwind::set_hook();

let res: Result<_, maybe_unwind::UnwindContext> = maybe_unwind(|| do_something());
if let Err(err) = res {
    eprintln!("cause: {:?}", err.cause);
    eprintln!("captured: {:?}", err.captured);
}

maybe_unwind::reset_hook();
# fn do_something() {}
```
!*/

#![cfg_attr(feature = "nightly", feature(backtrace))]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]

use cfg_if::cfg_if;
use lazy_static::lazy_static;
use std::{
    any::Any,
    cell::Cell,
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
#[inline]
pub fn set_hook() {
    match PREV_HOOK.write() {
        Ok(mut prev_hook) => {
            prev_hook.get_or_insert_with(|| {
                let prev_hook = panic::take_hook();
                panic::set_hook(Box::new(maybe_unwind_panic_hook));
                prev_hook
            });
        }
        Err(err) => panic!("{}", err),
    }
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
    static CAPTURED_PANIC_INFO: Cell<Option<NonNull<Option<CapturedPanicInfo>>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<Option<CapturedPanicInfo>>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        CAPTURED_PANIC_INFO.with(|captured| {
            captured.set(self.0.take());
        });
    }
}

fn maybe_unwind_panic_hook(info: &PanicInfo) {
    let captured = CAPTURED_PANIC_INFO.with(|captured| captured.replace(None));
    let _reset = SetOnDrop(captured);

    match captured {
        Some(mut captured) => unsafe {
            let captured = captured.as_mut();
            captured.replace(CapturedPanicInfo {
                payload: {
                    let payload = info.payload();
                    payload
                        .downcast_ref::<&str>()
                        .map(|payload| payload.to_string())
                        .or_else(|| payload.downcast_ref::<String>().cloned())
                },
                message: info.to_string(),
                file: info.location().map(|loc| loc.file().to_string()),
                line: info.location().map(|loc| loc.line()),
                column: info.location().map(|loc| loc.column()),
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

/// An information about a panic.
#[derive(Debug)]
pub struct CapturedPanicInfo {
    payload: Option<String>,
    message: String,
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
    #[cfg(feature = "nightly")]
    backtrace: Backtrace,
}

impl CapturedPanicInfo {
    /// Return the payload associated with the panic.
    #[inline]
    pub fn payload(&self) -> Option<&str> {
        self.payload.as_deref()
    }

    /// Return the formatted panic message.
    #[inline]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    /// Return the name of the source file from which the panic originated.
    #[inline]
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    /// Return the line number from which the panic originated.
    #[inline]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    /// Return the column from which the panic originated.
    #[inline]
    pub fn column(&self) -> Option<u32> {
        self.column
    }

    /// Return the captured backtrace for the panic.
    #[cfg(feature = "nightly")]
    #[doc(cfg(feature = "nightly"))]
    #[inline]
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

/// The values captured due to a panic.
#[derive(Debug)]
#[non_exhaustive]
pub struct UnwindContext {
    /// The cause of unwinding panic, caught by `catch_unwind`.
    pub cause: Box<dyn Any + Send + 'static>,

    /// The panic information, caught in the panic hook.
    pub captured: Option<CapturedPanicInfo>,
}

/// Invokes a closure, capturing the cause of an unwinding panic if one occurs.
///
/// In addition, this function also captures the panic information if the custom
/// panic hook is set. If the panic hook is not set, only the cause of unwinding
/// panic captured by `catch_unwind` is returned.
///
/// See also the documentation of [`CapturedPanicInfo`] for details.
///
/// [`CapturedPanicInfo`]: ./struct.CapturedPanicInfo.html
pub fn maybe_unwind<F, R>(f: F) -> Result<R, UnwindContext>
where
    F: FnOnce() -> R + UnwindSafe,
{
    let mut captured: Option<CapturedPanicInfo> = None;

    let res = {
        let old_info = CAPTURED_PANIC_INFO.with(|tls| {
            tls.replace(Some(NonNull::from(&mut captured))) //
        });
        let _reset = SetOnDrop(old_info);
        panic::catch_unwind(f)
    };

    res.map_err(|cause| UnwindContext {
        cause,
        captured: captured.take(),
    })
}
