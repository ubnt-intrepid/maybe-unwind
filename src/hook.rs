use crate::{
    tls::Context,
    unwind::{Captured, Location},
};
use std::panic::PanicInfo;

#[cfg(feature = "nightly")]
use std::backtrace::Backtrace;

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
