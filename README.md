# `maybe-unwind`

A variant of [`catch_unwind`](https://docs.rs/futures/0.3/futures/future/trait.FutureExt.html#method.catch_unwind) that also captures the panic information.

The purpose of this library is to provide a utility for capturing the error information from assetion macros in custom test libraries.

## Status

WIP

## Example

```rust
use maybe_unwind::FutureExt;

// Replace the global panic handler so that the panic information can be captured.
let old_hook = maybe_uninit::set_hook();

let res: Result<_, maybe_unwind::UnwindContext> = 
    do_something_that_may_panic()
        .maybe_unwind()
        .await;

if let Err(err) = res {
    eprintln!("cause: {:?}", err.cause);
    eprintln!("error_msg: {}", err.message);
}

std::panic::set_hook(old_hook);
```

## License

This library is licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
