const LIB_DIR: &str = "lib";
const INCLUDE_DIR: &str = "include";

fn main() {
    let mut conf = cmake::Config::new("mfem");
    conf.define("BUILD_LIBRARY_TYPE", "Static")
        .define("INSTALL_DIR_LIB", LIB_DIR)
        .define("INSTALL_DIR_INCLUDE", INCLUDE_DIR);
    #[cfg(not(feature = "precision-f32"))]
    conf.define("MFEM_PRECISION", "double");
    #[cfg(feature = "precision-f32")]
    conf.define("MFEM_PRECISION", "single");
    let dst = conf.build();

    println!("cargo:rustc-env=MFEM_PATH={}", dst.display());
}
