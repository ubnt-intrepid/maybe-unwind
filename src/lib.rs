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

#![doc(html_root_url = "https://docs.rs/maybe-unwind/0.3.1")]
#![deny(missing_docs)]
#![forbid(clippy::todo, clippy::unimplemented)]
#![cfg_attr(backtrace, feature(backtrace))]
#![cfg_attr(docs, feature(doc_cfg))]

#[macro_use]
mod backtrace;
#[macro_use]
mod context;
mod hook;
mod unwind;

pub use crate::{
    hook::capture_panic_info,
    unwind::{maybe_unwind, Location, Unwind},
};

#[cfg(feature = "futures")]
mod futures;

#[cfg(feature = "futures")]
pub use futures::{FutureMaybeUnwindExt, MaybeUnwind};
