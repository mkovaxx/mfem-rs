const LIB_DIR: &str = "lib";
const INCLUDE_DIR: &str = "include";

fn main() {
    let dst = cmake::Config::new("mfem")
        .define("BUILD_LIBRARY_TYPE", "Static")
        .define("INSTALL_DIR_LIB", LIB_DIR)
        .define("INSTALL_DIR_INCLUDE", INCLUDE_DIR)
        .build();

    println!(
        "cargo:rustc-env=MFEM_PATH={}",
        dst.to_str().expect("path is valid Unicode")
    );
}
