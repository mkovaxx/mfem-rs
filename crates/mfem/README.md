# mfem

Part of `mfem-rs`, a Rust wrapper for [MFEM](https://mfem.org/).

## ⚠️ Work in Progress ⚠️

This crate is currently under heavy development and as such, is likely unstable.  
Please file an issue and bear with us while we sort things out! Thanks! :)

## About

- Hides sharp bits such as `UniquePtr`, C/C++ strings, etc.
- Turns constructor-like FFI functions into `Self::new()`, etc.
- Turns method-like FFI functions into real `.method()`s.
- Provides field setters and getters.
- Turns C++ base classes into traits.
- Has identifiers that follow Rust best practices.
- Depends on `mfem-sys`.
