use std::ffi::CStr;

use clap::Parser;
use cxx::{let_cxx_string, UniquePtr};
use mfem_sys::ffi::{
    ArrayInt_SetAll, ArrayInt_ctor, ArrayInt_ctor_size, BasisType, ConstantCoefficient_ctor,
    FiniteElementSpace_GetEssentialTrueDofs, FiniteElementSpace_ctor, GridFunction_OwnFEC,
    H1_FECollection, H1_FECollection_as_fec, H1_FECollection_ctor, LinearForm_ctor_fes,
    Mesh_GetNodes, Mesh_bdr_attributes, Mesh_ctor_file, OrderingType,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Mesh file to use.
    #[arg(short, long = "mesh", value_name = "FILE")]
    mesh_file: String,

    /// Finite element order (polynomial degree) or -1 for isoparametric space.
    #[arg(short, long, default_value_t = 1)]
    order: i32,
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
    let mut mesh = Mesh_ctor_file(&mesh_file, 1, 1, true);
    let dim = mesh.Dimension();

    dbg!(mesh.GetNE());

    // 4. Refine the mesh to increase the resolution. In this example we do
    //    'ref_levels' of uniform refinement. We choose 'ref_levels' to be the
    //    largest number that gives a final mesh with no more than 50,000
    //    elements.
    {
        let ref_levels = f64::floor(f64::log2(50000.0 / mesh.GetNE() as f64) / dim as f64) as u32;

        for _ in 0..ref_levels {
            mesh.pin_mut().UniformRefinement(0);
        }
    }

    dbg!(mesh.GetNE());

    // 5. Define a finite element space on the mesh. Here we use continuous
    //    Lagrange finite elements of the specified order. If order < 1, we
    //    instead use an isoparametric/isogeometric space.
    let owned_fec: Option<UniquePtr<H1_FECollection>> = if args.order > 0 {
        Some(H1_FECollection_ctor(
            args.order,
            dim,
            BasisType::GaussLobatto.repr,
        ))
    } else if Mesh_GetNodes(&mesh).is_err() {
        Some(H1_FECollection_ctor(1, dim, BasisType::GaussLobatto.repr))
    } else {
        None
    };

    let fec = match &owned_fec {
        Some(ptr) => H1_FECollection_as_fec(&ptr),
        None => {
            println!("Using isoparametric FEs");
            let nodes = Mesh_GetNodes(&mesh).expect("Mesh has its own nodes");
            let iso_fec = GridFunction_OwnFEC(nodes).expect("OwnFEC exists");
            iso_fec
        }
    };

    unsafe {
        let name_ptr = fec.Name();
        assert!(!name_ptr.is_null());
        let fec_name = CStr::from_ptr(name_ptr);
        dbg!(fec_name);
    }

    let fespace = FiniteElementSpace_ctor(&mesh, fec, 1, OrderingType::byNODES);
    println!(
        "Number of finite element unknowns: {}",
        fespace.GetTrueVSize(),
    );

    let mut ess_tdof_list = ArrayInt_ctor();
    if Mesh_bdr_attributes(&mesh).Size() > 0 {
        let mut ess_bdr = ArrayInt_ctor_size(Mesh_bdr_attributes(&mesh).Max());
        ArrayInt_SetAll(ess_bdr.pin_mut(), 1);
        FiniteElementSpace_GetEssentialTrueDofs(&fespace, &ess_bdr, ess_tdof_list.pin_mut(), -1);
    }

    let b = LinearForm_ctor_fes(&fespace);
    let one = ConstantCoefficient_ctor(1.0);
    // b.AddDomainIntegrator(DomainLFIntegrator_ctor(one));
    // b.Assemble();
}
