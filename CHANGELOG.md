# Changelog

All notable changes to this project will be documented in this file.

This format is based on [Keep a Changelog], and this project adheres to [Semantic Versioning].

## [Unreleased]

## [0.2.0] (2020-02-25)

### Fixed

* check if the thread is panicking before modifying the hook ([9ed3dcc](https://github.com/ubnt-intrepid/maybe-unwind/commit/9ed3dcc7eaea9e01fdd079bce37bbdd78fee305b))

### Breaking Changes

* call the original hook regardless of where the panic occurred ([0e1392f](https://github.com/ubnt-intrepid/maybe-unwind/commit/0e1392fcc516737d6c0d497e655524760d474464))

## [0.1.2] (2020-02-02)

### Fixed

* add note about data race when setting panic hook ([0865aab](https://github.com/ubnt-intrepid/maybe-unwind/commit/0865aabfc0ac7a7929923f9580230efc92bf6549))

## [0.1.1] (2020-01-27)

### Added

* implementation of `fmt::Display` for `Unwind` ([79f1b0e](https://github.com/ubnt-intrepid/maybe-unwind/commit/79f1b0e47237e4b113053fc15120ce0b454dc2ec))

## [0.1.0] (2020-01-27)

### Added

* Add `Location` representing the location information where the panic originated ([f16e17e](https://github.com/ubnt-intrepid/maybe-unwind/commit/f16e17ec66a6f4853b5b28e7dafdb85fb2105023))

### Changed

* Remove `Unwind::{file,line,column}` (use `Unwind::location` instead)

## [0.0.2] (2020-01-26)

### Changed

* Refine captured panic information ([9095d30](https://github.com/ubnt-intrepid/maybe-unwind/commit/9095d30a6b29b3608f8c599c4fe4c2ef6d04e583))

## [0.0.1] (2020-01-26)

* initial release

<!-- links -->

[Unreleased]: https://github.com/ubnt-intrepid/maybe-unwind/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ubnt-intrepid/maybe-unwind/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/ubnt-intrepid/maybe-unwind/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ubnt-intrepid/maybe-unwind/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ubnt-intrepid/maybe-unwind/compare/v0.0.2...v0.1.0
[0.0.2]: https://github.com/ubnt-intrepid/maybe-unwind/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/ubnt-intrepid/maybe-unwind/tree/v0.0.1

[Keep a Changelog]: https://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
