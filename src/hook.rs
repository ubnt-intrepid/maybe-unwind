use crate::{
    tls::{AccessError, Context},
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
/// use maybe_unwind::{maybe_unwind, set_hook};
///
/// set_hook();
///
/// let res = maybe_unwind(|| { panic!("oops"); });
/// assert!(res.is_err());
/// ```
#[inline]
pub fn set_hook() {
    if thread::panicking() {
        panic!("cannot modify the panic hook from a panicking thread");
    }

    let mut prev_hook = PREV_HOOK.write().unwrap();
    prev_hook.get_or_insert_with(|| {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|info| {
            if Context::is_set() {
                if let Err(_access_err) = capture_panic_info(info) {
                    eprintln!("warning: failed to capture the panic information");
                }
            }

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

#[inline(never)]
pub fn capture_panic_info(info: &PanicInfo) -> Result<(), AccessError> {
    #[cfg(feature = "nightly")]
    let backtrace = Backtrace::capture();

    Context::try_with(|ctx| {
        ctx.captured.replace(Captured {
            location: info.location().map(|loc| Location::from_std(loc)),
            #[cfg(feature = "nightly")]
            backtrace,
        });
    })
}
