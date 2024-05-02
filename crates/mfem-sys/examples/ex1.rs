use std::ffi::CStr;

use clap::Parser;
use cxx::{let_cxx_string, UniquePtr};
use mfem_sys::ffi::{
    ArrayInt_SetAll, ArrayInt_ctor, ArrayInt_ctor_size, BasisType,
    BilinearForm_AddDomainIntegrator, BilinearForm_FormLinearSystem, BilinearForm_ctor_fes,
    ConstantCoefficient_as_coeff, ConstantCoefficient_ctor, DiffusionIntegrator_ctor,
    DiffusionIntegrator_into_bfi, DomainLFIntegrator_ctor_ab, DomainLFIntegrator_into_lfi,
    FiniteElementSpace_GetEssentialTrueDofs, FiniteElementSpace_ctor, GSSmoother_as_mut_Solver,
    GSSmoother_ctor, GridFunction_OwnFEC, GridFunction_SetAll, GridFunction_as_mut_Vector,
    GridFunction_as_vector, GridFunction_ctor_fes, H1_FECollection, H1_FECollection_as_fec,
    H1_FECollection_ctor, LinearForm_AddDomainIntegrator, LinearForm_as_vector,
    LinearForm_ctor_fes, Mesh_GetNodes, Mesh_bdr_attributes, Mesh_ctor_file, OperatorHandle_as_ref,
    OperatorHandle_ctor, OperatorHandle_try_as_SparseMatrix, OrderingType, Vector_ctor, PCG,
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

    // 6. Determine the list of true (i.e. conforming) essential boundary dofs.
    //    In this example, the boundary conditions are defined by marking all
    //    the boundary attributes from the mesh as essential (Dirichlet) and
    //    converting them to a list of true dofs.
    let mut ess_tdof_list = ArrayInt_ctor();
    if Mesh_bdr_attributes(&mesh).Size() > 0 {
        let mut ess_bdr = ArrayInt_ctor_size(Mesh_bdr_attributes(&mesh).Max());
        ArrayInt_SetAll(ess_bdr.pin_mut(), 1);
        FiniteElementSpace_GetEssentialTrueDofs(&fespace, &ess_bdr, ess_tdof_list.pin_mut(), -1);
    }

    // 7. Set up the linear form b(.) which corresponds to the right-hand side of
    //    the FEM linear system, which in this case is (1,phi_i) where phi_i are
    //    the basis functions in the finite element fespace.
    let mut b = LinearForm_ctor_fes(&fespace);
    let one = ConstantCoefficient_ctor(1.0);
    let one_coeff = ConstantCoefficient_as_coeff(&one);
    let integrator = DomainLFIntegrator_ctor_ab(one_coeff, 2, 0);
    let lfi = DomainLFIntegrator_into_lfi(integrator);
    LinearForm_AddDomainIntegrator(b.pin_mut(), lfi);
    b.pin_mut().Assemble();

    // 8. Define the solution vector x as a finite element grid function
    //    corresponding to fespace. Initialize x with initial guess of zero,
    //    which satisfies the boundary conditions.
    let mut x = GridFunction_ctor_fes(&fespace);
    GridFunction_SetAll(x.pin_mut(), 0.0);

    // 9. Set up the bilinear form a(.,.) on the finite element space
    //    corresponding to the Laplacian operator -Delta, by adding the Diffusion
    //    domain integrator.
    let mut a = BilinearForm_ctor_fes(&fespace);
    let bf_integrator = DiffusionIntegrator_ctor(one_coeff);
    let bfi = DiffusionIntegrator_into_bfi(bf_integrator);
    BilinearForm_AddDomainIntegrator(a.pin_mut(), bfi);

    // 10. Assemble the bilinear form and the corresponding linear system,
    //     applying any necessary transformations such as: eliminating boundary
    //     conditions, applying conforming constraints for non-conforming AMR,
    //     static condensation, etc.
    a.pin_mut().Assemble(0);

    let mut a_mat = OperatorHandle_ctor();
    let mut b_vec = Vector_ctor();
    let mut x_vec = Vector_ctor();
    BilinearForm_FormLinearSystem(
        &a,
        &ess_tdof_list,
        GridFunction_as_vector(&x),
        LinearForm_as_vector(&b),
        a_mat.pin_mut(),
        x_vec.pin_mut(),
        b_vec.pin_mut(),
    );

    println!(
        "Size of linear system: {}",
        OperatorHandle_as_ref(&a_mat).Height()
    );

    dbg!(a_mat.Type());

    // 11. Solve the linear system A X = B.
    // Use a simple symmetric Gauss-Seidel preconditioner with PCG.
    let a_sparse = OperatorHandle_try_as_SparseMatrix(&a_mat).expect("Operator is a SparseMatrix");
    let mut m_mat = GSSmoother_ctor(a_sparse, 0, 1);
    let solver = GSSmoother_as_mut_Solver(m_mat.pin_mut());
    PCG(
        OperatorHandle_as_ref(&a_mat),
        solver,
        &b_vec,
        x_vec.pin_mut(),
        1,
        200,
        1e-12,
        0.0,
    );

    // 12. Recover the solution as a finite element grid function.
    a.pin_mut().RecoverFEMSolution(
        &x_vec,
        LinearForm_as_vector(&b),
        GridFunction_as_mut_Vector(x.pin_mut()),
    );
}
