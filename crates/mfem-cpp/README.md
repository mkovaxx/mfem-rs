# mfem-cpp

Part of `mfem-rs`, a Rust wrapper for [MFEM](https://mfem.org/).

## ⚠️ Work in Progress ⚠️

This crate is currently under heavy development and as such, is likely unstable. Please file an issue and bear with us while we sort things out! Thanks! :)

## About

- If you just want to use MFEM from Rust, depend on the `mfem` crate instead.
- Provides a specific version of MFEM (currently 4.6.0).
- The version of MFEM is part of the package version, e.g.  
  `mfem-cpp = "0.1.0+mfem-4.6.0"`
- Has a feature called `bundled`.
  - on: Build MFEM from bundled source code.
  - off: Find (with CMake) MFEM installed on the system.
- The `lib.rs` provides `mfem_path()` to be used by `mfem-sys`.

## Credits

- The nifty CMake setup was copied from [opencascade-rs](https://github.com/bschwind/opencascade-rs).
