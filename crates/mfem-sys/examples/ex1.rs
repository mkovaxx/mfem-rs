use std::ffi::CStr;

use clap::Parser;
use cxx::{let_cxx_string, UniquePtr};
use mfem_sys::ffi::{
    BasisType, FiniteElementCollection_Name, FiniteElementSpace_ctor, GridFunction_OwnFEC,
    H1_FECollection, H1_FECollection_as_fec, H1_FECollection_ctor, Mesh_Dimension, Mesh_GetNE,
    Mesh_GetNodes, Mesh_UniformRefinement, Mesh_ctor_file, OrderingType,
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
    let dim = Mesh_Dimension(&mesh);

    dbg!(Mesh_GetNE(&mesh));

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

    dbg!(Mesh_GetNE(&mesh));

    // 5. Define a finite element space on the mesh. Here we use continuous
    //    Lagrange finite elements of the specified order. If order < 1, we
    //    instead use an isoparametric/isogeometric space.
    let mut owned_fec: Option<UniquePtr<H1_FECollection>> = None;
    if args.order > 0 {
        owned_fec = Some(H1_FECollection_ctor(
            args.order,
            dim,
            BasisType::GaussLobatto.repr,
        ));
    } else if Mesh_GetNodes(&mesh).is_err() {
        owned_fec = Some(H1_FECollection_ctor(1, dim, BasisType::GaussLobatto.repr));
    }

    let fec = match &owned_fec {
        Some(ptr) => H1_FECollection_as_fec(&ptr),
        None => unsafe {
            println!("Using isoparametric FEs");
            let nodes = Mesh_GetNodes(&mesh).expect("Mesh has its own nodes");
            let iso_fec = GridFunction_OwnFEC(nodes).as_ref().expect("OwnFEC exists");
            iso_fec
        },
    };

    unsafe {
        let name_ptr = FiniteElementCollection_Name(fec);
        assert!(!name_ptr.is_null());
        let fec_name = CStr::from_ptr(name_ptr);
        dbg!(fec_name);
    }

    let fespace = FiniteElementSpace_ctor(mesh.pin_mut(), fec, 1, OrderingType::byNODES);
    // println!(
    //     "Number of finite element unknowns: {}",
    //     FESpace_GetTrueVSize(&fespace)
    // );
}
