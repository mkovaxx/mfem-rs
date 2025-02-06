use std::process::exit;

fn main() {
    let target = std::env::var("TARGET").expect("No TARGET environment variable defined");
    let is_windows = target.to_lowercase().contains("windows");

    let mut mfem_config = MfemConfig::detect();

    println!(
        "cargo:rustc-link-search=native={}",
        mfem_config.library_dir.display()
    );

    for lib in mfem_config.mfem_libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    if is_windows {
        println!("cargo:rustc-link-lib=dylib=user32");
    }

    mfem_config.include_dirs.push("src".into());
    mfem_config.include_dirs.push("include".into()); // for cxx.
    let mut build = autocxx_build::Builder::new("src/lib.rs", &mfem_config.include_dirs)
        .build()
        .expect("autocxx builder");
    build
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-O3")
        .flag_if_supported("-w");
    for f in mfem_config.cxx_flags {
        build.flag_if_supported(&f);
    }
    if let "windows" = std::env::consts::OS {
        let current = std::env::current_dir().unwrap();
        build.include(current.parent().unwrap());
    }

    build.compile("mfem-sys");

    println!("cargo:rustc-link-lib=static=mfem-sys");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ffi_autocxx.hpp");
    println!("cargo:rerun-if-changed=src/ffi_cxx.hpp");
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

        println!("cargo:rerun-if-env-changed=MFEM_DIR");

        // Add path to bundled MFEM
        #[cfg(feature = "bundled")]
        {
            std::env::set_var("MFEM_DIR", mfem_cpp::mfem_path().as_os_str());
        }

        let dst =
            std::panic::catch_unwind(|| cmake::Config::new("MFEM").register_dep("mfem").build());

        #[cfg(feature = "bundled")]
        let dst = dst.expect("Bundled MFEM not found.");

        #[cfg(not(feature = "bundled"))]
        let dst = dst.expect("Pre-installed MFEM not found.  You can use MFEM_DIR to point to its location. Alternatively, you can use `bundled` feature if you do not want to install MFEM system-wide.");

        let cfg = std::fs::read_to_string(dst.join("share").join("mfem_info.txt"))
            .expect("Something went wrong when detecting MFEM.");

        let mut version: Option<semver::Version> = None;
        let mut mfem_config = MfemConfig {
            mfem_libs: vec![],
            include_dirs: vec![],
            library_dir: std::path::PathBuf::new(),
            cxx_flags: vec![],
        };

        for line in cfg.lines() {
            if let Some((var, val)) = line.split_once('=') {
                match var {
                    // Keep in sync with "MFEM/CMakeLists.txt".
                    "VERSION" => version = semver::Version::parse(val).ok(),
                    "MFEM_LIBRARIES" => {
                        for l in val.split(";") {
                            mfem_config.mfem_libs.push(l.into());
                        }
                    }
                    "INCLUDE_DIRS" => {
                        for d in val.split(";") {
                            mfem_config.include_dirs.push(d.into());
                        }
                    }
                    "LIBRARY_DIR" => mfem_config.library_dir = val.into(),
                    "CXX_FLAGS" => {
                        for f in val.split(" ") {
                            mfem_config.cxx_flags.push(f.into());
                        }
                    }
                    "MFEM_USE_DOUBLE" => {
                        if !val.eq_ignore_ascii_case("on") {
                            println!("cargo:error=MFEM must be compiled with double precision");
                            exit(1);
                        }
                    }
                    _ => (),
                }
            }
        }

        if let Some(version) = version {
            if !version_req.matches(&version) {
                #[cfg(feature = "bundled")]
                panic!("Bundled MFEM found but version is not met (found {} but {} required). Please fix MFEM_VERSION in build script of `mfem-sys` crate or submodule mfem in `mfem-cpp` crate.",
                       version, version_req);

                #[cfg(not(feature = "bundled"))]
                panic!("Pre-installed MFEM found but version is not met (found {} but {} required). Please provide required version or use `bundled` feature.",
                       version, version_req);
            }
            mfem_config
        } else {
            panic!("MFEM found but something went wrong during configuration.");
        }
    }
}
