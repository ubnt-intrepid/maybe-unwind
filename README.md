<h1 align="center">
  <code>maybe-unwind</code>
</h1>
<div align="center">
  <strong>
    A variant of <a href="https://doc.rust-lang.org/stable/std/panic/fn.catch_unwind.html"><code>catch_unwind</code></a> that also captures the panic information.
  </strong>
</div>

<br />

<div align="center">
  <a href="https://crates.io/crates/maybe-unwind">
    <img src="https://img.shields.io/crates/v/maybe-unwind.svg?style=flat-square"
         alt="crates.io"
    />
  </a>
  <a href="https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html">
    <img src="https://img.shields.io/badge/rust-1.40.0-gray?style=flat-square"
         alt="rust toolchain"
    />
  </a>
  <!--
  <a href="https://docs.rs/mimicaw">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
         alt="docs.rs" />
  </a>
  -->
</div>

<br />

The main purpose of this library is to provide a utility for capturing the error information from assetion macros in custom test libraries.

## License

This library is licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

