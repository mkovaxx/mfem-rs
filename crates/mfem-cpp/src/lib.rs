use std::path::Path;

/// Get the path to the MFEM library installation directory to be
/// used in build scripts.
///
/// Only valid during build (`cargo clean` removes these files).
pub fn mfem_path() -> &'static Path {
    Path::new(env!("MFEM_PATH"))
}
