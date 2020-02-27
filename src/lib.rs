/*!
A wrapper of [`catch_unwind`] that also captures the panic information.

The main purpose of this library is to provide a utility for capturing
the error information from assetion macros in custom test libraries.

[`catch_unwind`]: https://doc.rust-lang.org/stable/std/panic/fn.catch_unwind.html

# Example

```
use maybe_unwind::maybe_unwind;

std::panic::set_hook(Box::new(|info| {
    maybe_unwind::capture_panic_info(info);
}));

if let Err(unwind) = maybe_unwind(|| do_something()) {
    eprintln!("payload = {:?}", unwind.payload());
    eprintln!("location = {:?}", unwind.location());
}
# fn do_something() {}
```
!*/

#![doc(html_root_url = "https://docs.rs/maybe-unwind/0.2.0")]
#![deny(missing_docs)]
#![forbid(clippy::todo, clippy::unimplemented)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(feature = "nightly", feature(backtrace))]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![doc(test(attr(deny(deprecated))))]

mod hook;
mod tls;
mod unwind;

pub use crate::{
    hook::capture_panic_info,
    unwind::{maybe_unwind, Location, Unwind},
};

#[allow(deprecated)]
pub use crate::hook::{reset_hook, set_hook};

cfg_if::cfg_if! {
    if #[cfg(feature = "futures")] {
        mod futures;
        pub use futures::{FutureMaybeUnwindExt, MaybeUnwind};
    }
}
