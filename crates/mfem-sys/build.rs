/// Minimum compatible version of MFEM (major, minor)
///
/// Pre-installed MFEM will be checked for compatibility using semver rules.
const MFEM_VERSION: (u8, u8) = (4, 6);

/// The list of used MFEM libraries which needs to be linked with.
const MFEM_LIBS: &[&str] = &["MFEM"];

fn main() {
    let target = std::env::var("TARGET").expect("No TARGET environment variable defined");
    let is_windows = target.to_lowercase().contains("windows");

    let mfem_config = MfemConfig::detect();

    println!(
        "cargo:rustc-link-search=native={}",
        mfem_config.library_dir.to_str().unwrap()
    );

    let lib_type = if mfem_config.is_dynamic {
        "dylib"
    } else {
        "static"
    };
    for lib in MFEM_LIBS {
        println!("cargo:rustc-link-lib={lib_type}={lib}");
    }

    if is_windows {
        println!("cargo:rustc-link-lib=dylib=user32");
    }

    let mut build = cxx_build::bridge("src/lib.rs");

    if let "windows" = std::env::consts::OS {
        let current = std::env::current_dir().unwrap();
        build.include(current.parent().unwrap());
    }

    build
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .include(mfem_config.include_dir)
        .include("include")
        .compile("wrapper");

    println!("cargo:rustc-link-lib=static=wrapper");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/wrapper.hpp");
}

struct MfemConfig {
    include_dir: std::path::PathBuf,
    library_dir: std::path::PathBuf,
    is_dynamic: bool,
}

impl MfemConfig {
    /// Find MFEM using cmake
    fn detect() -> Self {
        println!("cargo:rerun-if-env-changed=DEP_MFEM_ROOT");

        // Add path to bundled MFEM
        #[cfg(feature = "bundled")]
        {
            std::env::set_var("DEP_MFEM_ROOT", mfem_cpp::mfem_path().as_os_str());
        }

        let dst =
            std::panic::catch_unwind(|| cmake::Config::new("MFEM").register_dep("mfem").build());

        #[cfg(feature = "bundled")]
        let dst = dst.expect("Bundled MFEM not found.");

        #[cfg(not(feature = "bundled"))]
        let dst = dst.expect("Pre-installed MFEM not found. You can use `bundled` feature if you do not want to install MFEM system-wide.");

        let cfg = std::fs::read_to_string(dst.join("share").join("mfem_info.txt"))
            .expect("Something went wrong when detecting MFEM.");

        let mut version_major: Option<u8> = None;
        let mut version_minor: Option<u8> = None;
        let mut include_dir: Option<std::path::PathBuf> = None;
        let mut library_dir: Option<std::path::PathBuf> = None;
        let mut is_dynamic: bool = false;

        for line in cfg.lines() {
            if let Some((var, val)) = line.split_once('=') {
                match var {
                    "VERSION_MAJOR" => version_major = val.parse().ok(),
                    "VERSION_MINOR" => version_minor = val.parse().ok(),
                    "INCLUDE_DIR" => include_dir = val.parse().ok(),
                    "LIBRARY_DIR" => library_dir = val.parse().ok(),
                    "BUILD_SHARED_LIBS" => is_dynamic = val == "ON",
                    _ => (),
                }
            }
        }

        if let (Some(version_major), Some(version_minor), Some(include_dir), Some(library_dir)) =
            (version_major, version_minor, include_dir, library_dir)
        {
            if version_major != MFEM_VERSION.0 || version_minor < MFEM_VERSION.1 {
                #[cfg(feature = "bundled")]
                panic!("Bundled MFEM found but version is not met (found {}.{} but {}.{} required). Please fix MFEM_VERSION in build script of `mfem-sys` crate or submodule mfem in `mfem-cpp` crate.",
                       version_major, version_minor, MFEM_VERSION.0, MFEM_VERSION.1);

                #[cfg(not(feature = "bundled"))]
                panic!("Pre-installed MFEM found but version is not met (found {}.{} but {}.{} required). Please provide required version or use `bundled` feature.",
                       version_major, version_minor, MFEM_VERSION.0, MFEM_VERSION.1);
            }

            Self {
                include_dir,
                library_dir,
                is_dynamic,
            }
        } else {
            panic!("MFEM found but something went wrong during configuration.");
        }
    }
}
