# mfem-sys

Part of `mfem-rs`, a Rust wrapper for [MFEM](https://mfem.org/).

## ⚠️ Work in Progress ⚠️

This crate is currently under heavy development and as such, is likely unstable.  
Please file an issue and bear with us while we sort things out! Thanks! :)

## About

- If you just want to use MFEM from Rust, depend on the `mfem` crate instead.
- This crate is very low level and thus cumbersom to use.
- A safe FFI (foreign-function interface) to use MFEM from Rust.
- Uses the `cxx` crate to generate safe and correct bindings.
- Encodes MFEM's ownership rules into Rust's type system.
- Turns various MFEM `int` constants into type-safe Rust `enum`s.
- Optionally depends on `mfem-cpp`: if the C++ libraries `libmfem` is
  not installed on you system (with the development files, e.g. via
  the package `libmfem-dev` on Debian), activate the feature `bundled`
  so that `libmfem` is compiled in your project.
