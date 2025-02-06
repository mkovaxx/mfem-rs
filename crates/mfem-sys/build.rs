/// The list of used MFEM libraries which needs to be linked with.
fn main() {
    let target = std::env::var("TARGET").expect("No TARGET environment variable defined");
    let is_windows = target.to_lowercase().contains("windows");

    let mfem_config = MfemConfig::detect();

    println!(
        "cargo:rustc-link-search=native={}",
        mfem_config.library_dir.to_str().unwrap()
    );

    for lib in mfem_config.mfem_libs {
        #[cfg(feature = "bundled")]
        println!("cargo:rustc-link-lib=static={lib}");
        #[cfg(not(feature = "bundled"))]
        println!("cargo:rustc-link-lib={lib}");
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
        .includes(mfem_config.include_dirs)
        .include("include");
    for f in mfem_config.cxx_flags {
        build.flag_if_supported(&f);
    }

    build.compile("wrapper");

    println!("cargo:rustc-link-lib=static=wrapper");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/wrapper.hpp");
}

#[derive(Debug)]
struct MfemConfig {
    mfem_libs: Vec<String>,
    include_dirs: Vec<std::path::PathBuf>,
    library_dir: std::path::PathBuf,
    cxx_flags: Vec<String>,
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
        let mut mfem_libs: Vec<String> = vec![];
        let mut include_dirs: Vec<std::path::PathBuf> = vec![];
        let mut library_dir: Option<std::path::PathBuf> = None;
        let mut cxx_flags: Vec<String> = vec![];

        for line in cfg.lines() {
            if let Some((var, val)) = line.split_once('=') {
                match var {
                    // Keep in sync with "MFEM/CMakeLists.txt".
                    "VERSION" => version = semver::Version::parse(val).ok(),
                    "MFEM_LIBRARIES" => {
                        for l in val.split(" ") {
                            // FIXME: Right delim?
                            mfem_libs.push(l.into());
                        }
                    }
                    "INCLUDE_DIRS" => {
                        for d in val.split(";") {
                            include_dirs.push(d.into());
                        }
                    }
                    "LIBRARY_DIR" => library_dir = val.parse().ok(),
                    "CXX_FLAGS" => {
                        for f in val.split(" ") {
                            cxx_flags.push(f.into());
                        }
                    }
                    _ => (),
                }
            }
        }

        if let (Some(version), Some(library_dir)) = (version, library_dir) {
            if !version_req.matches(&version) {
                #[cfg(feature = "bundled")]
                panic!("Bundled MFEM found but version is not met (found {} but {} required). Please fix MFEM_VERSION in build script of `mfem-sys` crate or submodule mfem in `mfem-cpp` crate.",
                       version, version_req);

                #[cfg(not(feature = "bundled"))]
                panic!("Pre-installed MFEM found but version is not met (found {} but {} required). Please provide required version or use `bundled` feature.",
                       version, version_req);
            }

            Self {
                mfem_libs,
                include_dirs,
                library_dir,
                cxx_flags,
            }
        } else {
            panic!("MFEM found but something went wrong during configuration.");
        }
    }
}
