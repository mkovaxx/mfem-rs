# mfem-rs

Rust wrapper for [MFEM](https://mfem.org/).

## ⚠️ Work in Progress ⚠️

This crate is currently under heavy development and as such, is likely unstable. Please file an issue and bear with us while we sort things out! Thanks! :)

## About

This is a monorepo with the following 3 `cargo` packages in a workspace:
- `mfem`  
  Wraps `mfem-sys` to support writing idiomatic Rust.
- `mfem-sys`  
  Binds (via `cxx`) to `mfem-cpp` and encodes ownership rules.
- `mfem-cpp`  
  Provides the C++ MFEM library as a `cargo` package.

See the `README.md` file of each crate for more information.
