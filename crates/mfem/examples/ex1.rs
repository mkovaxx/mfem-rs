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
use mfem::*;

fn main() -> anyhow::Result<()> {
    // 1. Parse command-line options.
    let args = Args::parse();

    // 2. Enable hardware devices such as GPUs, and programming models such as
    //    CUDA, OCCA, RAJA and OpenMP based on command line options.
    // TODO(mkovaxx)

    // 3. Read the mesh from the given mesh file. We can handle triangular,
    //    quadrilateral, tetrahedral, hexahedral, surface and volume meshes with
    //    the same code.
    let mut mesh = Mesh::from_file(&args.mesh_file)?;
    let dim = mesh.dimension();
    dbg!(dim);
    dbg!(mesh.get_num_elems());

    // 4. Refine the mesh to increase the resolution. In this example we do
    //    'ref_levels' of uniform refinement. We choose 'ref_levels' to be the
    //    largest number that gives a final mesh with no more than 50,000
    //    elements.
    let ref_levels =
        f64::floor(f64::log2(50000.0 / mesh.get_num_elems() as f64) / dim as f64) as u32;
    for _ in 0..ref_levels {
        mesh.uniform_refinement(RefAlgo::A);
    }
    dbg!(mesh.get_num_elems());

    // 5. Define a finite element space on the mesh. Here we use continuous
    //    Lagrange finite elements of the specified order. If order < 1, we
    //    instead use an isoparametric/isogeometric space.
    let owned_fec: Option<H1FeCollection> = if args.order > 0 {
        Some(H1FeCollection::new(
            args.order,
            dim,
            BasisType::GaussLobatto,
        ))
    } else if mesh.get_nodes().is_none() {
        Some(H1FeCollection::new(1, dim, BasisType::GaussLobatto))
    } else {
        None
    };

    let owned_nodes = mesh.get_nodes();

    let fec: &dyn FiniteElementCollection = match &owned_fec {
        Some(h1_fec) => h1_fec,
        None => {
            println!("Using isoparametric FEs");
            let nodes = owned_nodes.as_ref().expect("Mesh has its own nodes");
            let iso_fec = nodes.get_own_fec().expect("OwnFEC exists");
            iso_fec
        }
    };

    dbg!(fec.get_name());

    let fespace = FiniteElementSpace::new(&mesh, fec, 1, OrderingType::byNODES);
    println!(
        "Number of finite element unknowns: {}",
        fespace.get_true_vsize(),
    );

    // 6. Determine the list of true (i.e. conforming) essential boundary dofs.
    //    In this example, the boundary conditions are defined by marking all
    //    the boundary attributes from the mesh as essential (Dirichlet) and
    //    converting them to a list of true dofs.
    let mut ess_tdof_list = ArrayInt::new();
    if let Some(max_bdr_attr) = mesh.get_bdr_attributes().iter().max() {
        let mut ess_bdr = ArrayInt::with_len(*max_bdr_attr as usize);
        ess_bdr.set_all(1);
        fespace.get_essential_true_dofs(&ess_bdr, &mut ess_tdof_list, None);
    }

    // 7. Set up the linear form b(.) which corresponds to the right-hand side of
    //    the FEM linear system, which in this case is (1,phi_i) where phi_i are
    //    the basis functions in the finite element fespace.
    let mut b = LinearForm::new(&fespace);
    let one = ConstantCoefficient::new(1.0);
    let integrator = DomainLFIntegrator::new(&one, 2, 0);
    b.add_domain_integrator(integrator);
    b.assemble();

    Ok(())
}
