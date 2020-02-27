use crate::{
    tls::Context,
    unwind::{Captured, Location},
};
use lazy_static::lazy_static;
use std::{
    panic::{self, PanicInfo},
    sync::RwLock,
    thread,
};

type PanicHook = dyn Fn(&PanicInfo) + Send + Sync + 'static;

lazy_static! {
    static ref PREV_HOOK: RwLock<Option<Box<PanicHook>>> = RwLock::new(None);
}

/// Registers the custom panic hook so that the panic information can be captured.
///
/// This function saves the current panic hook and replaces with a custom hook that
/// captures the panic information caused by closures enclosed in [`maybe_unwind`].
/// After capturing the panic information, the original panic hook is *always* called
/// regardless of where the panic occurred.
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
/// # #![allow(deprecated)]
/// use maybe_unwind::{maybe_unwind, set_hook};
///
/// set_hook();
///
/// let res = maybe_unwind(|| { panic!("oops"); });
/// assert!(res.is_err());
/// ```
#[deprecated(since = "0.2.1")]
#[inline]
pub fn set_hook() {
    if thread::panicking() {
        panic!("cannot modify the panic hook from a panicking thread");
    }

    let mut prev_hook = PREV_HOOK.write().unwrap();
    prev_hook.get_or_insert_with(|| {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|info| {
            capture_panic_info(info);

            let prev_hook = PREV_HOOK.read().ok();
            let prev_hook = prev_hook.as_ref().and_then(|prev_hook| prev_hook.as_ref());
            if let Some(prev_hook) = prev_hook {
                (prev_hook)(info);
            } else {
                eprintln!("warning: the original panic hook is not available (this is a bug).");
            }
        }));
        prev_hook
    });
}

/// Unregisters the custom panic hook and reset the previous hook.
#[deprecated(since = "0.2.1")]
#[inline]
pub fn reset_hook() {
    if thread::panicking() {
        panic!("cannot modify the panic hook from a panicking thread");
    }

    if let Ok(mut prev_hook) = PREV_HOOK.write() {
        if let Some(prev_hook) = prev_hook.take() {
            panic::set_hook(prev_hook);
        }
    }
}

/// Capture the panic information.
///
/// The captured values are stored in the thread local context
/// for passing to the caller of `maybe_unwind`. After capturing
/// the panic information, this function returns `true`.
///
/// If the panic location is outside of the closure passed to
/// `maybe_unwind`, this function does nothing and just return
/// `false`.
///
/// # Example
///
/// ```
/// use maybe_unwind::{maybe_unwind, capture_panic_info};
/// use std::panic::{self, PanicInfo};
///
/// fn my_hook(info: &PanicInfo) {
///     let captured = capture_panic_info(info);
///
///     if !captured {
///         println!("{}", info);
///     }
/// }
/// panic::set_hook(Box::new(my_hook));
///
/// let res = maybe_unwind(|| { panic!("oops"); });
/// assert!(res.is_err());
/// ```
pub fn capture_panic_info(info: &PanicInfo) -> bool {
    if !Context::is_set() {
        return false;
    }

    #[cfg(feature = "nightly")]
    let backtrace = Backtrace::capture();

    let _ = Context::try_with(|ctx| {
        ctx.captured.replace(Captured {
            location: info.location().map(|loc| Location::from_std(loc)),
            #[cfg(feature = "nightly")]
            backtrace,
        });
    });

    true
}
