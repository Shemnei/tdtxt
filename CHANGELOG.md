# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Fuzzing case for parsing of a `Task`
- Generic `From<S: Into<String>>` implemented for `Description`
- Added functions `State::is_done` and `State::is_open`
- Added function `Priority::as_char`
- Added struct `Component` and `Components` as iterator return value for the new function `Description::components`
- Added parse errors to the public API

### Changed

- `Description::new` now accepts `Into<String>` instead of `ToString`

## [0.2.0] - 2021-09-01

### Added

- Created `prelude` module for quick importing of all important features
- The crate now exports all useful definitions in the root
- Added enum variant `State::Open`
- Implemented `std::std::FromStr` for all task components
- Added rustdoc documentation to the public api
- Added feature `chrono` for optional support for `chrono::NaiveDate` dates
- Added feature `serde` for optional support for de(serializing) todo.txt components
- Added example `examples/filter_open.rs`
- Added `PartialOrd` and `Ord` for `SimpleDate`, `Date`, `DateCompound` and `State`

### Changed

- Functions `Description::projects()`, `Description::contexts()` and `Description::custom()` now return an `Iterator` instead of a `Vec`
- Replaced `Range<usize>` with the struct `ByteSpan`
- Renamed `Todo` to `Task`
- Field `Task::state` is now `State` instead of `Option<State>`
- Renamed `XXXParseError` to `ParseXXXError` to follow the convention of the std lib

## [0.1.0] - 2021-08-26

### Added

- Basic parsing capabilities

[Unreleased]: https://github.com/Shemnei/tdtxt/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Shemnei/tdtxt/releases/tag/v0.2.0
[0.1.0]: https://github.com/Shemnei/tdtxt/releases/tag/v0.1.0
