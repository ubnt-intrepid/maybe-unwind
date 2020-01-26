/*!
A wrapper of [`catch_unwind`] that also captures the panic information.

The main purpose of this library is to provide a utility for capturing
the error information from assetion macros in custom test libraries.

[`catch_unwind`]: https://doc.rust-lang.org/stable/std/panic/fn.catch_unwind.html

# Example

```no_run
use maybe_unwind::maybe_unwind;

maybe_unwind::set_hook();

if let Err(unwind) = maybe_unwind(|| do_something()) {
    eprintln!("payload = {:?}", unwind.payload());
    eprintln!("file = {:?}", unwind.file());
    eprintln!("line = {:?}", unwind.line());
    eprintln!("column = {:?}", unwind.column());
}
# fn do_something() {}
```
!*/

#![doc(html_root_url = "https://docs.rs/maybe-unwind/0.0.2")]
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
    static OWNED_PANIC_INFO: Cell<Option<NonNull<Option<OwnedPanicInfo>>>> = Cell::new(None);
}

struct SetOnDrop(Option<NonNull<Option<OwnedPanicInfo>>>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        OWNED_PANIC_INFO.with(|info| {
            info.set(self.0.take());
        });
    }
}

fn maybe_unwind_panic_hook(raw_info: &PanicInfo) {
    let info = OWNED_PANIC_INFO.with(|info| info.replace(None));
    let _reset = SetOnDrop(info);

    match info {
        Some(mut info) => unsafe {
            let info = info.as_mut();
            info.replace(OwnedPanicInfo {
                file: raw_info.location().map(|loc| loc.file().to_string()),
                line: raw_info.location().map(|loc| loc.line()),
                column: raw_info.location().map(|loc| loc.column()),
                #[cfg(feature = "nightly")]
                backtrace: Backtrace::capture(),
            });
        },
        None => fallback_hook(raw_info),
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
    info: Option<OwnedPanicInfo>,
}

#[derive(Debug)]
struct OwnedPanicInfo {
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
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

    /// Return the name of the source file from which the panic originated.
    #[inline]
    pub fn file(&self) -> Option<&str> {
        self.info.as_ref()?.file.as_deref()
    }

    /// Return the line number from which the panic originated.
    #[inline]
    pub fn line(&self) -> Option<u32> {
        self.info.as_ref()?.line
    }

    /// Return the column from which the panic originated.
    #[inline]
    pub fn column(&self) -> Option<u32> {
        self.info.as_ref()?.column
    }

    /// Return the captured backtrace for the panic.
    #[cfg(feature = "nightly")]
    #[doc(cfg(feature = "nightly"))]
    #[inline]
    pub fn backtrace(&self) -> Option<&Backtrace> {
        Some(&self.info.as_ref()?.backtrace)
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
    let mut info: Option<OwnedPanicInfo> = None;

    let res = {
        let old_info = OWNED_PANIC_INFO.with(|tls| {
            tls.replace(Some(NonNull::from(&mut info))) //
        });
        let _reset = SetOnDrop(old_info);
        panic::catch_unwind(f)
    };

    res.map_err(|payload| Unwind {
        payload,
        info: info.take(),
    })
}
