/// The list of used MFEM libraries which needs to be linked with.
const MFEM_LIBS: &[&str] = &["mfem"];

fn main() {
    let target = std::env::var("TARGET").expect("No TARGET environment variable defined");
    let is_windows = target.to_lowercase().contains("windows");

    let mfem_config = MfemConfig::detect();

    println!(
        "cargo:rustc-link-search=native={}",
        mfem_config.library_dir.to_str().unwrap()
    );

    let lib_type = "static";
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
        .std("c++14")
        .flag_if_supported("-w")
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
}

impl MfemConfig {
    /// Find MFEM using cmake
    fn detect() -> Self {
        // Minimum compatible version of MFEM
        //
        // Pre-installed MFEM will be checked for compatibility using semver rules.
        let version_req = semver::VersionReq::parse(">=4.6").expect("Invalid version constraint");

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

        let mut version: Option<semver::Version> = None;
        let mut include_dir: Option<std::path::PathBuf> = None;
        let mut library_dir: Option<std::path::PathBuf> = None;

        for line in cfg.lines() {
            if let Some((var, val)) = line.split_once('=') {
                match var {
                    "VERSION" => version = semver::Version::parse(val).ok(),
                    "INCLUDE_DIR" => include_dir = val.parse().ok(),
                    "LIBRARY_DIR" => library_dir = val.parse().ok(),
                    _ => (),
                }
            }
        }

        if let (Some(version), Some(include_dir), Some(library_dir)) =
            (version, include_dir, library_dir)
        {
            if !version_req.matches(&version) {
                #[cfg(feature = "bundled")]
                panic!("Bundled MFEM found but version is not met (found {} but {} required). Please fix MFEM_VERSION in build script of `mfem-sys` crate or submodule mfem in `mfem-cpp` crate.",
                       version, version_req);

                #[cfg(not(feature = "bundled"))]
                panic!("Pre-installed MFEM found but version is not met (found {} but {} required). Please provide required version or use `bundled` feature.",
                       version, MFEM_VERSION);
            }

            Self {
                include_dir,
                library_dir,
            }
        } else {
            panic!("MFEM found but something went wrong during configuration.");
        }
    }
}
