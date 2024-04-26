use clap::Parser;
use cxx::let_cxx_string;
use mfem_sys::ffi::{Mesh_Dimension, Mesh_GetNE, Mesh_UniformRefinement, Mesh_ctor_file};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Mesh file to use
    #[arg(short, long = "mesh", value_name = "FILE")]
    mesh_file: String,
}

fn main() {
    // 1. Parse command-line options.
    let args = Args::parse();

    // 2. Enable hardware devices such as GPUs, and programming models such as
    //    CUDA, OCCA, RAJA and OpenMP based on command line options.
    // TODO(mkovaxx)

    // 3. Read the mesh from the given mesh file. We can handle triangular,
    //    quadrilateral, tetrahedral, hexahedral, surface and volume meshes with
    //    the same code.
    let_cxx_string!(mesh_file = args.mesh_file);
    let mut mesh = Mesh_ctor_file(&mesh_file);
    let dim = Mesh_Dimension(&mesh);

    println!("mesh.GetNE(): {}", Mesh_GetNE(&mesh));

    // 4. Refine the mesh to increase the resolution. In this example we do
    //    'ref_levels' of uniform refinement. We choose 'ref_levels' to be the
    //    largest number that gives a final mesh with no more than 50,000
    //    elements.
    {
        let ref_levels =
            f64::floor(f64::log2(50000.0 / Mesh_GetNE(&mesh) as f64) / dim as f64) as u32;

        for _ in 0..ref_levels {
            Mesh_UniformRefinement(mesh.pin_mut(), 0);
        }
    }

    println!("mesh.GetNE(): {}", Mesh_GetNE(&mesh));
}
