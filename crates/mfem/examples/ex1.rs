/// MFEM Example 1
///
/// This example code demonstrates the use of MFEM to define a
/// simple finite element discretization of the Laplace problem
/// -Delta u = 1 with homogeneous Dirichlet boundary conditions.
/// Specifically, we discretize using a FE space of the specified
/// order, or if order < 1 using an isoparametric/isogeometric
/// space (i.e. quadratic for quadratic curvilinear mesh, NURBS for
/// NURBS mesh, etc.)
///
/// The example highlights the use of mesh refinement, finite
/// element grid functions, as well as linear and bilinear forms
/// corresponding to the left-hand side and right-hand side of the
/// discrete linear system. We also cover the explicit elimination
/// of essential boundary conditions, static condensation, and the
/// optional connection to the GLVis tool for visualization.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Mesh file to use.
    #[arg(short, long = "mesh", value_name = "FILE")]
    mesh_file: String,

    /// Finite element order (polynomial degree) or -1 for isoparametric space.
    #[arg(short, long, default_value_t = 1)]
    order: i32,
}

use clap::Parser;
use mfem::Mesh;

fn main() -> anyhow::Result<()> {
    // 1. Parse command-line options.
    let args = Args::parse();

    // 2. Enable hardware devices such as GPUs, and programming models such as
    //    CUDA, OCCA, RAJA and OpenMP based on command line options.
    // TODO(mkovaxx)

    // 3. Read the mesh from the given mesh file. We can handle triangular,
    //    quadrilateral, tetrahedral, hexahedral, surface and volume meshes with
    //    the same code.
    let mesh = Mesh::from_file(&args.mesh_file)?;
    dbg!(mesh.dimension());
    dbg!(mesh.get_num_elems());

    Ok(())
}
