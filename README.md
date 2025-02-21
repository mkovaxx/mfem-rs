# mfem-rs

Rust wrapper for [MFEM](https://mfem.org/).

## ⚠️ Work in Progress ⚠️

This crate is currently under heavy development and as such, is likely unstable.  
Please file an issue and bear with us while we sort things out! Thanks! :)

## About

This is a monorepo with the following 3 `cargo` packages in a workspace:
- `mfem`  
  Wraps `mfem-sys` to support writing idiomatic Rust.
- `mfem-sys`  
  Binds (via `cxx`) to `mfem-cpp` and encodes ownership rules.
- `mfem-cpp`  
  Provides the C++ MFEM library as a `cargo` package.

See the `README.md` file of each crate for more information.

## Talks

Talk at the MFEM Community Workshop 2024:

[![MFEM Workshop 2024 | Rust Wrapper](https://img.youtube.com/vi/4X8Q06kKcFA/0.jpg)](https://www.youtube.com/watch?v=4X8Q06kKcFA)

Longer and more detailed talk at the [Tokyo Rust](https://www.tokyorust.org) meetup:

[![MFEM Workshop 2024 | Rust Wrapper](https://img.youtube.com/vi/2xBVQczO8_Y/0.jpg)](https://www.youtube.com/watch?v=2xBVQczO8_Y)
