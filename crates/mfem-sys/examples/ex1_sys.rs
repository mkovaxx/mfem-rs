#![allow(non_snake_case)]

/// MFEM Example 1
///
/// This example code demonstrates the use of MFEM to define a
/// simple finite element discretization of the Laplace problem
/// -Δu = 1 with homogeneous Dirichlet boundary conditions.
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

use std::{ffi::CStr, pin::Pin, ptr, slice};

use autocxx::prelude::*;
use clap::Parser;
use cxx::{let_cxx_string, UniquePtr};
use mfem_sys::*;

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
    let mut mesh = UniquePtr::emplace(Mesh::LoadFromFile(&mesh_file, c_int(1), c_int(1), true));
    let dim = mesh.Dimension();

    dbg!(mesh.GetNE());

    // 4. Refine the mesh to increase the resolution. In this example we do
    //    'ref_levels' of uniform refinement. We choose 'ref_levels' to be the
    //    largest number that gives a final mesh with no more than 50,000
    //    elements.
    {
        let ne: i32 = mesh.GetNE().into();
        let dim: i32 = dim.into();
        let ref_levels = ((50000.0 / ne as f64).log2() / dim as f64).floor() as u32;

        for _ in 0..ref_levels {
            mesh.pin_mut().UniformRefinement1(c_int(0));
        }
    }

    dbg!(mesh.GetNE());

    // 5. Define a finite element space on the mesh. Here we use continuous
    //    Lagrange finite elements of the specified order. If order < 1, we
    //    instead use an isoparametric/isogeometric space.
    let owned_fec: Option<UniquePtr<H1_FECollection>> = if args.order > 0 {
        Some(UniquePtr::emplace(H1_FECollection::new(
            c_int(args.order),
            dim,
            c_int(BasisType::GaussLobatto as i32),
        )))
    } else if !mesh.GetNodes2().is_null() {
        Some(UniquePtr::emplace(H1_FECollection::new(
            c_int(1),
            dim,
            c_int(BasisType::GaussLobatto as i32),
        )))
    } else {
        None
    };

    let fec = match &owned_fec {
        Some(ptr) => {
            let fec = ptr.as_ref().unwrap();
            // `H1_FECollection` is a subclass of `FiniteElementCollection`.
            unsafe { std::mem::transmute::<&H1_FECollection, &FiniteElementCollection>(fec) }
        }
        None => {
            println!("Using isoparametric FEs");
            let nodes: *const GridFunction = mesh.GetNodes2();
            assert!(!nodes.is_null());
            let iso_fec = GridFunction_OwnFEC(unsafe { &*nodes });
            assert!(!iso_fec.is_null());
            unsafe { &*iso_fec }
        }
    };

    unsafe {
        let name_ptr = fec.Name();
        assert!(!name_ptr.is_null());
        let fec_name = CStr::from_ptr(name_ptr);
        dbg!(fec_name);
    }

    let fespace = FES_new(mesh.pin_mut(), fec, c_int(1), Ordering_Type::byNODES);
    println!(
        "Number of finite element unknowns: {}",
        fespace.GetTrueVSize(),
    );

    // 6. Determine the list of true (i.e. conforming) essential boundary dofs.
    //    In this example, the boundary conditions are defined by marking all
    //    the boundary attributes from the mesh as essential (Dirichlet) and
    //    converting them to a list of true dofs.
    let mut ess_tdof_list = arrayint_with_len(0);
    let bdr_attr = Mesh_bdr_attributes(&mesh);
    if Into::<i32>::into(bdr_attr.Size()) > 0 {
        let &n = slice_of_array_int(&bdr_attr).iter().max().unwrap();
        let mut ess_bdr = arrayint_with_len(n);
        slice_mut_of_array_int(ess_bdr.pin_mut()).fill(1);
        fespace.GetEssentialTrueDofs(&ess_bdr, ess_tdof_list.pin_mut(), -1);
    }

    // 7. Set up the linear form b(.) which corresponds to the right-hand side of
    //    the FEM linear system, which in this case is (1,phi_i) where phi_i are
    //    the basis functions in the finite element fespace.
    let mut b = UniquePtr::emplace(unsafe { LinearForm::new1(fespace.as_mut_ptr()) });
    let one = UniquePtr::emplace(ConstantCoefficient::new(1.0));
    let mut one = ConstantCoefficient_into_Coefficient(one);
    let lfi = UniquePtr::emplace(DomainLFIntegrator::new(one.pin_mut(), c_int(2), c_int(0)));
    // Beware that the linear form takes ownership of `lfi`.
    unsafe { b.pin_mut().AddDomainIntegrator(lfi.into_raw() as *mut _) };
    b.pin_mut().Assemble();

    // 8. Define the solution vector x as a finite element grid function
    //    corresponding to fespace. Initialize x with initial guess of zero,
    //    which satisfies the boundary conditions.
    let mut x = UniquePtr::emplace(unsafe { GridFunction::new2(fespace.as_mut_ptr()) });
    slice_mut_of_Vector(GridFunction_as_mut_Vector(x.pin_mut())).fill(0.0);

    // 9. Set up the bilinear form a(.,.) on the finite element space
    //    corresponding to the Laplacian operator -Delta, by adding the Diffusion
    //    domain integrator.
    let mut a = UniquePtr::emplace(unsafe { BilinearForm::new2(fespace.as_mut_ptr()) });
    let ir: *const IntegrationRule = ptr::null();
    let bfi = UniquePtr::emplace(unsafe { DiffusionIntegrator::new1(one.pin_mut(), ir) });
    // The bilinear form takes ownership of `bfi`.
    unsafe { a.pin_mut().AddDomainIntegrator(bfi.into_raw() as *mut _) };

    // 10. Assemble the bilinear form and the corresponding linear system,
    //     applying any necessary transformations such as: eliminating boundary
    //     conditions, applying conforming constraints for non-conforming AMR,
    //     static condensation, etc.
    a.pin_mut().Assemble(c_int(0));

    let mut a_mat = UniquePtr::emplace(OperatorHandle::new());
    let mut b_vec = UniquePtr::emplace(Vector::new());
    let mut x_vec = UniquePtr::emplace(Vector::new());
    a.pin_mut().FormLinearSystem(
        &ess_tdof_list,
        GridFunction_as_mut_Vector(x.pin_mut()),
        LinearForm_as_mut_Vector(b.pin_mut()),
        a_mat.pin_mut(),
        x_vec.pin_mut(),
        b_vec.pin_mut(),
        c_int(0),
    );

    println!(
        "Size of linear system: {}",
        OperatorHandle_oper(a_mat.as_ref().unwrap()).Height()
    );

    dbg!(a_mat.Type());

    // 11. Solve the linear system A X = B.
    // Use a simple symmetric Gauss-Seidel preconditioner with PCG.
    let a_sparse = unsafe { OperatorHandle_ref_SparseMatrix(&a_mat) };
    let mut m_mat = UniquePtr::emplace(GSSmoother::new1(a_sparse, c_int(0), c_int(1)));
    let solver = GSSmoother_as_mut_Solver(m_mat.pin_mut());
    PCG(
        OperatorHandle_oper(&a_mat),
        solver,
        &b_vec,
        x_vec.pin_mut(),
        1,
        200,
        1e-12.into(),
        0.0.into(),
    );
    // 12. Recover the solution as a finite element grid function.
    a.pin_mut().RecoverFEMSolution(
        &x_vec,
        LinearForm_as_Vector(&b),
        GridFunction_as_mut_Vector(x.pin_mut()),
    );

    // 13. Save the refined mesh and the solution. This output can be viewed later
    //     using GLVis: "glvis -m refined.mesh -g sol.gf".
    let_cxx_string!(mesh_filename = "refined.mesh");
    mesh.Save(&mesh_filename, c_int(8));
    let_cxx_string!(sol_filename = "sol.gf");
    unsafe { x.Save1(sol_filename.as_ptr() as *const _, c_int(8)) };
}

fn slice_of_array_int(a: &ArrayInt) -> &[i32] {
    let len: i32 = a.Size().into();
    let data = a.GetData() as *const i32;
    unsafe { slice::from_raw_parts(data, len as usize) }
}

fn slice_mut_of_array_int(a: Pin<&mut ArrayInt>) -> &mut [i32] {
    let len: i32 = a.Size().into();
    let data = a.GetDataMut() as *mut i32;
    unsafe { slice::from_raw_parts_mut(data, len as usize) }
}

fn ConstantCoefficient_into_Coefficient(
    c: UniquePtr<ConstantCoefficient>,
) -> UniquePtr<Coefficient> {
    unsafe { std::mem::transmute::<UniquePtr<ConstantCoefficient>, UniquePtr<Coefficient>>(c) }
}

fn LinearForm_as_Vector(x: &LinearForm) -> &Vector {
    // LinearForm is a subclass of Vector.
    unsafe { &*mfem_sys::LinearForm_as_Vector(x as *const _) }
}

fn LinearForm_as_mut_Vector(x: Pin<&mut LinearForm>) -> Pin<&mut Vector> {
    // LinearForm is a subclass of Vector.
    unsafe {
        Pin::new_unchecked(&mut *mfem_sys::LinearForm_as_mut_Vector(
            x.get_unchecked_mut(),
        ))
    }
}

fn GridFunction_as_mut_Vector(gf: Pin<&mut GridFunction>) -> Pin<&mut Vector> {
    // GridFunction is a subclass of Vector.
    unsafe {
        Pin::new_unchecked(&mut *mfem_sys::GridFunction_as_mut_Vector(
            gf.get_unchecked_mut(),
        ))
    }
}

fn slice_mut_of_Vector(v: Pin<&mut Vector>) -> &mut [real_t] {
    let len: i32 = v.Size().into();
    let data = v.GetData();
    unsafe { slice::from_raw_parts_mut(data, len as usize) }
}

fn GSSmoother_as_mut_Solver(s: Pin<&mut GSSmoother>) -> Pin<&mut Solver> {
    // GSSmoother is a subclass of Solver.
    unsafe { std::mem::transmute::<Pin<&mut GSSmoother>, Pin<&mut Solver>>(s) }
}
