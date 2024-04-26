use clap::Parser;
use cxx::let_cxx_string;
use mfem_sys::ffi::{Mesh_GetNE, Mesh_ctor_file};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Mesh file to use
    #[arg(short, long = "mesh", value_name = "FILE")]
    mesh_file: String,
}

fn main() {
    let args = Args::parse();

    let_cxx_string!(filename = args.mesh_file);
    let mesh = Mesh_ctor_file(&filename);

    println!("NE: {}", Mesh_GetNE(&mesh));
}
