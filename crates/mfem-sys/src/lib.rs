#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::all)]
#![allow(
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_html_tags,
    rustdoc::bare_urls
)]

use autocxx::prelude::*;

include_cpp! {
    // "mfem.hpp" is included by ffi_autocxx.hpp with some pragma to
    // disable spurious warnings.
    #include "ffi_autocxx.hpp"
    safety!(unsafe)
    generate_pod!("mfem::real_t")
    generate!("acxx::MFEM_USE_EXCEPTIONS")
    generate!("acxx::NumBasisTypes")
    generate!("mfem::ErrorAction")
    generate!("mfem::set_error_action")
    generate!("mfem::Vector")

    extern_cpp_opaque_type!("mfem::Operator", ffi_cxx::Operator)
    generate!("mfem::Operator_Type") // Operator::Type
    generate!("mfem::Matrix")

    generate!("mfem::Element_Type")

    generate!("mfem::Mesh")
    generate!("acxx::Mesh_bdr_attributes")
    generate!("mfem::Mesh_Operation")       // Mesh::Operation
    generate!("mfem::Mesh_FaceTopology")    // Mesh::FaceTopology
    generate!("mfem::Mesh_ElementLocation") // Mesh::ElementLocation
    generate!("mfem::Mesh_ElementConformity") // Mesh::ElementConformity
    generate!("mfem::Mesh_FaceInfoTag")     // Mesh::FaceInfoTag

    generate!("mfem::FiniteElement")
    generate!("mfem::FiniteElement_MapType")
    generate!("mfem::FiniteElementCollection")
    generate!("mfem::L2_FECollection")
    generate!("mfem::H1_FECollection")
    generate!("mfem::H1_Trace_FECollection")
    generate!("mfem::RT_FECollection")
    generate!("mfem::ND_FECollection")
    generate!("mfem::CrouzeixRaviartFECollection")

    generate!("mfem::Ordering")
    // `FiniteElementSpace::VarOrderBits` is protected types which is not
    // handled well.  But Ok if generated indirectly.
    // generate!("mfem::FiniteElementSpace")
    generate!("acxx::FES_new")

    generate!("mfem::GridFunction")
    generate!("acxx::GridFunction_OwnFEC") // immutable version
    generate!("mfem::LinearFormIntegrator")
    generate!("mfem::BilinearFormIntegrator")
    generate!("mfem::LinearForm")
    generate!("mfem::BilinearForm")
    generate!("mfem::MixedBilinearForm")

    generate!("mfem::Coefficient")
    generate!("mfem::ConstantCoefficient")
    generate!("mfem::FunctionCoefficient")
    generate!("mfem::GridFunctionCoefficient")
    generate!("mfem::InnerProductCoefficient")
    generate!("mfem::VectorCoefficient")
    generate!("mfem::VectorConstantCoefficient")
    generate!("mfem::VectorFunctionCoefficient")

    generate!("mfem::OperatorHandle")
    generate!("mfem::LinearFormIntegrator")
    generate!("mfem::DeltaLFIntegrator")
    generate!("mfem::DomainLFIntegrator")
    generate!("mfem::BilinearFormIntegrator")
    generate!("mfem::DiffusionIntegrator")
    generate!("mfem::ConvectionIntegrator")

    generate!("mfem::SparseMatrix")
    generate!("mfem::Solver")
    generate!("mfem::GSSmoother")
    generate!("mfem::PermuteFaceL2")
}

// We handle abstract classes in this module.  One can convert
// pointers to these classes and use the methods.  This avoids writing
// C++ code to apply all the abstract methods to the sub-classes.
#[cxx::bridge]
mod ffi_cxx {
    unsafe extern "C++" {
        include!("ffi_cxx.hpp");

        #[namespace = "mfem"]
        #[cxx_name = "real_t"]
        type real = crate::Real;
        #[cxx_name = "mfem_Array_int_AutocxxConcrete"]
        type ArrayInt = crate::ffi::mfem_Array_int_AutocxxConcrete;
        #[cxx_name = "array_with_len"]
        fn arrayint_with_len(size: i32) -> UniquePtr<ArrayInt>;
        #[cxx_name = "array_copy"]
        fn arrayint_copy(src: &ArrayInt) -> UniquePtr<ArrayInt>;
        #[cxx_name = "array_from_slice"]
        unsafe fn arrayint_from_slice(
            data: *mut i32,
            len: i32,
            own_data: bool,
        ) -> UniquePtr<ArrayInt>;
        fn Size(self: &ArrayInt) -> i32;
        fn GetData(self: &ArrayInt) -> *const i32;
        #[cxx_name = "GetData"]
        fn GetDataMut(self: Pin<&mut ArrayInt>) -> *mut i32;

        // mfem::Operator::Type.  mfem::Operator is not really a
        // namespace but this is needed for the type Id to coincide with
        // autocxx.  (We want that compatibility because, say,
        // `OperatorHandle::Type` also return that type.)
        #[namespace = "mfem::Operator"]
        type Type = crate::Operator_Type;
        #[namespace = "mfem"]
        #[cxx_name = "Vector"]
        type VectorCxx = crate::Vector;
        type Operator;

        fn GetType(self: &Operator) -> Type;
        fn Height(self: &Operator) -> i32;
        fn Width(self: &Operator) -> i32;
        /// Safety: Virtual method.  Make sure it applied only to proper
        /// sub-classes
        unsafe fn RecoverFEMSolution(
            self: Pin<&mut Operator>,
            X: &VectorCxx,
            b: &VectorCxx,
            x: Pin<&mut VectorCxx>,
        );

        #[namespace = "mfem"]
        #[cxx_name = "Mesh"]
        type MeshCxx = crate::Mesh;
        #[namespace = "mfem"]
        #[cxx_name = "FiniteElementCollection"]
        type FiniteElementCollectionCxx = crate::FiniteElementCollection;

        #[namespace = "mfem"]
        #[cxx_name = "FiniteElementSpace"]
        type FiniteElementSpaceCxx = crate::FiniteElementSpace;
        fn Conforming(self: &FiniteElementSpaceCxx) -> bool;
        fn Nonconforming(self: &FiniteElementSpaceCxx) -> bool;
        fn GetEssentialVDofs(
            self: &FiniteElementSpaceCxx,
            bdr_attr_is_ess: &ArrayInt,
            ess_vdofs: Pin<&mut ArrayInt>,
            component: i32,
        );
        fn GetEssentialTrueDofs(
            self: &FiniteElementSpaceCxx,
            bdr_attr_is_ess: &ArrayInt,
            ess_tdof_list: Pin<&mut ArrayInt>,
            component: i32,
        );
        fn GetBoundaryTrueDofs(
            self: Pin<&mut FiniteElementSpaceCxx>,
            boundary_dofs: Pin<&mut ArrayInt>,
            component: i32,
        );
        fn GetTrueVSize(self: &FiniteElementSpaceCxx) -> i32;
        fn SetElementOrder(self: Pin<&mut FiniteElementSpaceCxx>, i: i32, p: i32);
        fn GetElementOrder(self: &FiniteElementSpaceCxx, i: i32) -> i32;
        fn IsVariableOrder(self: &FiniteElementSpaceCxx) -> bool;
        fn GetMesh(self: &FiniteElementSpaceCxx) -> *mut MeshCxx;
        fn GetVDim(self: &FiniteElementSpaceCxx) -> i32;
        fn GetNDofs(self: &FiniteElementSpaceCxx) -> i32;
        fn GetVSize(self: &FiniteElementSpaceCxx) -> i32;
        fn FEColl(self: &FiniteElementSpaceCxx) -> *const FiniteElementCollectionCxx;

        #[namespace = "mfem"]
        type Element;
        #[namespace = "mfem::Element"]
        #[cxx_name = "Type"]
        type Element_TypeCxx = crate::Element_Type;
        fn GetType(self: &Element) -> Element_TypeCxx;

        // autocxx does not bind any constructor of `FunctionCoefficient`.
        // Moreover, we "enhance" the interface to allow to pass closures.
        #[namespace = "mfem"]
        #[cxx_name = "FunctionCoefficient"]
        type FunctionCoefficientCxx = crate::FunctionCoefficient;
        type c_void;
        unsafe fn FunctionCoefficient_new(
            f: unsafe fn(&VectorCxx, data: *mut c_void) -> real,
            data: *mut c_void,
        ) -> UniquePtr<FunctionCoefficientCxx>;

        #[namespace = "mfem"]
        #[cxx_name = "Matrix"]
        type MatrixCxx = crate::Matrix;
        #[cxx_name = "upcast_to_operator"]
        fn Matrix_to_operator<'a>(m: &'a MatrixCxx) -> &'a Operator;
        #[cxx_name = "upcast_to_operator_mut"]
        fn Matrix_to_operator_mut<'a>(m: Pin<&'a mut MatrixCxx>) -> Pin<&'a mut Operator>;

        #[namespace = "mfem"]
        #[cxx_name = "OperatorHandle"]
        type OperatorHandleCxx = crate::OperatorHandle;
        fn OperatorHandle_operator<'a>(o: &'a OperatorHandleCxx) -> &'a Operator;
        fn OperatorHandle_operator_mut<'a>(
            o: Pin<&'a mut OperatorHandleCxx>,
        ) -> Pin<&'a mut Operator>;

        #[namespace = "mfem"]
        #[cxx_name = "SparseMatrix"]
        type SparseMatrixCxx = crate::SparseMatrix;
        unsafe fn OperatorHandle_ref_SparseMatrix<'a>(
            o: &'a OperatorHandleCxx,
        ) -> &'a SparseMatrixCxx;
        unsafe fn SparseMatrix_to_OperatorHandle<'a>(
            o: *mut SparseMatrixCxx,
        ) -> UniquePtr<OperatorHandleCxx>;

        #[allow(clippy::too_many_arguments)]
        #[namespace = "mfem"]
        #[cxx_name = "Solver"]
        type SolverCxx = crate::Solver;
        fn PCG(
            a: &Operator,
            solver: Pin<&mut SolverCxx>,
            b: &VectorCxx,
            x: Pin<&mut VectorCxx>,
            print_iter: i32,
            max_num_iter: i32,
            rtol: real,
            atol: real,
        );
    }
    impl UniquePtr<Operator> {}
}

// Import into scope all C++ symbols defined above.
pub use ffi::acxx::*;
pub use ffi::mfem::*;
pub use ffi_cxx::*;

use cxx::{type_id, ExternType};
use std::fmt::{Debug, Error, Formatter};

/// A wrapper for the floating point numbers of the precision of the
/// MFEM library.
// `real_t` is an alias from `autocxx`.
#[repr(transparent)]
pub struct Real(pub real_t);

unsafe impl ExternType for Real {
    type Id = type_id!("mfem::real_t");
    type Kind = cxx::kind::Trivial;
}

impl From<real_t> for Real {
    fn from(value: real_t) -> Self {
        Real(value)
    }
}

impl From<Real> for real_t {
    fn from(value: Real) -> Self {
        value.0
    }
}

impl Debug for Operator_Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use Operator_Type::*;
        write!(
            f,
            "{}",
            match self {
                ANY_TYPE => "ANY_TYPE",
                MFEM_SPARSEMAT => "MFEM_SPARSEMAT",
                Hypre_ParCSR => "Hypre_ParCSR",
                PETSC_MATAIJ => "PETSC_MATAIJ",
                PETSC_MATIS => "PETSC_MATIS",
                PETSC_MATSHELL => "PETSC_MATSHELL",
                PETSC_MATNEST => "PETSC_MATNEST",
                PETSC_MATHYPRE => "PETSC_MATHYPRE",
                PETSC_MATGENERIC => "PETSC_MATGENERIC",
                Complex_Operator => "Complex_Operator",
                MFEM_ComplexSparseMat => "MFEM_ComplexSparseMat",
                Complex_Hypre_ParCSR => "Complex_Hypre_ParCSR",
                Complex_DenseMat => "Complex_DenseMat",
                MFEM_Block_Matrix => "MFEM_Block_Matrix",
                MFEM_Block_Operator => "MFEM_Block_Operator",
            }
        )
    }
}

pub enum BasisType {
    //Invalid = -1,  // Removed, use Option<BasisType>
    /// Open type.
    GaussLegendre = 0,
    /// Closed type.
    GaussLobatto = 1,
    /// Bernstein polynomials.
    Positive = 2,
    /// Nodes: x_i = (i+1)/(n+1), i=0,...,n-1
    OpenUniform = 3,
    /// Nodes: x_i = i/(n-1),     i=0,...,n-1.
    ClosedUniform = 4,
    /// Nodes: x_i = (i+1/2)/n,   i=0,...,n-1.
    OpenHalfUniform = 5,
    /// Serendipity basis (squares / cubes).
    Serendipity = 6,
    /// Closed GaussLegendre.
    ClosedGL = 7,
    /// Integrated GLL indicator functions.
    IntegratedGLL = 8,
    // NumBasisTypes = 9, see test right after.
}

#[cfg(test)]
#[test]
fn test_num_basis_types() {
    assert_eq!(NumBasisTypes, 9);
}

impl TryFrom<c_int> for BasisType {
    type Error = i32;

    /// Try to convert the value into a [`BasisType`].  If it fails,
    /// it returns the number as an error.
    fn try_from(c_int(value): c_int) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BasisType::GaussLegendre),
            1 => Ok(BasisType::GaussLobatto),
            2 => Ok(BasisType::Positive),
            3 => Ok(BasisType::OpenUniform),
            4 => Ok(BasisType::ClosedUniform),
            5 => Ok(BasisType::OpenHalfUniform),
            6 => Ok(BasisType::Serendipity),
            7 => Ok(BasisType::ClosedGL),
            8 => Ok(BasisType::IntegratedGLL),
            _ => Err(value),
        }
    }
}
