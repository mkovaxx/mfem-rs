fn main() {
    println!("mfem_path: {}", mfem_cpp::mfem_path().to_str().unwrap());
}
