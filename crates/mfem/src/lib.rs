use autocxx::c_int;
use autocxx::prelude::Emplace;
use mfem_sys::Real;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::*;

use cxx::{let_cxx_string, UniquePtr};
use thiserror::Error;

trait ThinWrapper {
    type Inner;

    fn from_ref(r: &Self::Inner) -> &Self;
    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self;

    fn into_ref(&self) -> &Self::Inner;
    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner>;
}

//////////////
// ArrayInt //
//////////////

#[repr(transparent)]
pub struct OwnedArrayInt {
    inner: UniquePtr<mfem_sys::ArrayInt>,
}

impl OwnedArrayInt {
    pub fn new() -> Self {
        let inner = mfem_sys::arrayint_with_len(0);
        Self { inner }
    }

    pub fn with_len(len: usize) -> Self {
        let inner = mfem_sys::arrayint_with_len(len as i32);
        Self { inner }
    }
}

impl Deref for OwnedArrayInt {
    type Target = ArrayInt;

    fn deref(&self) -> &Self::Target {
        ArrayInt::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedArrayInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        ArrayInt::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct ArrayInt {
    inner: *mut mfem_sys::ArrayInt,
}

impl ThinWrapper for ArrayInt {
    type Inner = mfem_sys::ArrayInt;

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }
}

impl ArrayInt {
    pub fn set_all(&mut self, value: i32) {
        // TODO: mfem_sys::ArrayInt_SetAll(self.inner.pin_mut(), value);
        let slice: &mut [i32] = self.as_slice_mut();
        for entry in slice {
            *entry = value;
        }
    }

    pub fn as_slice(&self) -> &[i32] {
        unsafe {
            let data = self.into_ref().GetData();
            let size = self.into_ref().Size() as usize;
            std::slice::from_raw_parts(data, size)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [i32] {
        unsafe {
            let data = self.into_pin_mut().GetDataMut();
            let size = self.into_ref().Size() as usize;
            std::slice::from_raw_parts_mut(data, size)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &i32> {
        self.as_slice().iter()
    }
}

////////////
// Vector //
////////////

pub struct OwnedVector {
    inner: UniquePtr<mfem_sys::Vector>,
}

impl OwnedVector {
    pub fn new() -> Self {
        let inner = UniquePtr::emplace(mfem_sys::Vector::new());
        Self { inner }
    }
}

impl Deref for OwnedVector {
    type Target = Vector;

    fn deref(&self) -> &Self::Target {
        Vector::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedVector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Vector::from_pin_mut(self.inner.pin_mut())
    }
}

pub struct Vector {
    inner: *mut mfem_sys::Vector,
}

impl ThinWrapper for Vector {
    type Inner = mfem_sys::Vector;

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }
}

impl Vector {
    pub fn set_all(&mut self, value: Real) {
        mfem_sys::Vector_set_all(self.into_pin_mut(), value.into());
    }
}

//////////
// Mesh //
//////////

#[repr(transparent)]
pub struct OwnedMesh {
    inner: UniquePtr<mfem_sys::Mesh>,
}

impl OwnedMesh {
    pub fn new() -> Self {
        let inner = UniquePtr::emplace(mfem_sys::MeshCxx::new1());
        Self { inner }
    }

    pub fn from_file(path: &str) -> Self {
        let generate_edges = 1;
        let refine = 1;
        let fix_orientation = true;
        let_cxx_string!(mesh_path = path);
        let inner = UniquePtr::emplace(mfem_sys::Mesh::LoadFromFile(
            &mesh_path,
            c_int(generate_edges),
            c_int(refine),
            fix_orientation,
        ));
        Self { inner }
    }
}

impl Deref for OwnedMesh {
    type Target = Mesh;

    fn deref(&self) -> &Self::Target {
        Mesh::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedMesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Mesh::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct Mesh {
    inner: *mut mfem_sys::Mesh,
}

impl ThinWrapper for Mesh {
    type Inner = mfem_sys::Mesh;

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }
}

impl Mesh {
    pub fn dimension(&self) -> i32 {
        self.into_ref().Dimension().into()
    }

    pub fn get_num_elems(&self) -> i32 {
        self.into_ref().GetNE().into()
    }

    pub fn get_nodes(&self) -> Option<&GridFunction> {
        let grid_func = self.into_ref().GetNodes2().cast_mut();
        if !grid_func.is_null() {
            Some(GridFunction::from_ref(unsafe { &*grid_func }))
        } else {
            None
        }
    }

    pub fn get_bdr_attributes(&self) -> &ArrayInt {
        ArrayInt::from_ref(mfem_sys::Mesh_bdr_attributes(self.into_ref()))
    }

    pub fn uniform_refinement(&mut self, ref_algo: RefAlgo) {
        self.into_pin_mut()
            .UniformRefinement1(c_int(ref_algo as i32));
    }

    pub fn save_to_file(&self, path: &str, precision: i32) {
        let_cxx_string!(fname = path);
        self.into_ref().Save(&fname, c_int(precision));
    }
}

/// Refinement Algorithm
#[repr(i32)]
#[derive(Debug, Copy, Clone)]
pub enum RefAlgo {
    /// Algorithm "A"
    /// Currently used only for pure tetrahedral meshes.
    /// Produces elements with better quality
    A = 0,
    /// Algorithm "B"
    B = 1,
}

pub use mfem_sys::BasisType;

/////////////////////////////
// FiniteElementCollection //
/////////////////////////////

pub struct OwnedFiniteElementCollection {
    inner: UniquePtr<mfem_sys::FiniteElementCollection>,
}

impl Deref for OwnedFiniteElementCollection {
    type Target = FiniteElementCollection;

    fn deref(&self) -> &Self::Target {
        FiniteElementCollection::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedFiniteElementCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        FiniteElementCollection::from_pin_mut(self.inner.pin_mut())
    }
}

pub struct FiniteElementCollection {
    inner: *mut mfem_sys::FiniteElementCollection,
}

impl ThinWrapper for FiniteElementCollection {
    type Inner = mfem_sys::FiniteElementCollection;

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }
}

impl FiniteElementCollection {
    pub fn get_name(&self) -> String {
        let ptr = self.into_ref().Name();
        assert!(!ptr.is_null());
        let name = unsafe { std::ffi::CStr::from_ptr(ptr) };
        name.to_owned().into_string().expect("Valid string")
    }
}

/////////////////////
// H1_FECollection //
/////////////////////

pub struct OwnedH1FeCollection {
    inner: UniquePtr<mfem_sys::H1_FECollection>,
}

impl OwnedH1FeCollection {
    pub fn new(p: i32, dim: i32, btype: BasisType) -> Self {
        let inner = UniquePtr::emplace(mfem_sys::H1_FECollection::new(
            c_int(p),
            c_int(dim),
            c_int(btype as i32),
        ));
        Self { inner }
    }
}

impl Deref for OwnedH1FeCollection {
    type Target = H1FeCollection;

    fn deref(&self) -> &Self::Target {
        H1FeCollection::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedH1FeCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        H1FeCollection::from_pin_mut(self.inner.pin_mut())
    }
}

pub struct H1FeCollection {
    inner: *mut mfem_sys::H1_FECollection,
}

impl ThinWrapper for H1FeCollection {
    type Inner = mfem_sys::H1_FECollection;

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }
}

impl Deref for H1FeCollection {
    type Target = FiniteElementCollection;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for H1FeCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

////////////////////////
// FiniteElementSpace //
////////////////////////

pub use mfem_sys::Ordering_Type as OrderingType;

pub struct OwnedFiniteElementSpace {
    inner: UniquePtr<mfem_sys::FiniteElementSpace>,
}

impl OwnedFiniteElementSpace {
    pub fn new(
        mesh: &mut Mesh,
        fec: &FiniteElementCollection,
        vdim: i32,
        ordering: OrderingType,
    ) -> Self {
        let inner = mfem_sys::FES_new(mesh.into_pin_mut(), fec.into_ref(), c_int(vdim), ordering);
        Self { inner }
    }
}

impl Deref for OwnedFiniteElementSpace {
    type Target = FiniteElementSpace;

    fn deref(&self) -> &Self::Target {
        FiniteElementSpace::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedFiniteElementSpace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        FiniteElementSpace::from_pin_mut(self.inner.pin_mut())
    }
}

pub struct FiniteElementSpace {
    inner: *mut mfem_sys::FiniteElementSpace,
}

impl ThinWrapper for FiniteElementSpace {
    type Inner = mfem_sys::FiniteElementSpace;

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }
}

impl FiniteElementSpace {
    pub fn get_true_vsize(&self) -> i32 {
        self.into_ref().GetTrueVSize()
    }

    pub fn get_essential_true_dofs(
        &self,
        bdr_attr_is_ess: &ArrayInt,
        ess_tdof_list: &mut ArrayInt,
        component: Option<usize>,
    ) {
        self.into_ref().GetEssentialTrueDofs(
            bdr_attr_is_ess.into_ref(),
            ess_tdof_list.into_pin_mut(),
            component.map(|c| c as i32).unwrap_or(-1),
        );
    }
}

//////////////////
// GridFunction //
//////////////////

pub struct OwnedGridFunction {
    inner: UniquePtr<mfem_sys::GridFunction>,
}

impl OwnedGridFunction {
    pub fn new(fespace: &FiniteElementSpace) -> Self {
        let inner = UniquePtr::emplace(unsafe { mfem_sys::GridFunction::new2(fespace.inner) });
        Self { inner }
    }
}

pub struct GridFunction {
    inner: *mut mfem_sys::GridFunction,
}

impl ThinWrapper for GridFunction {
    type Inner = mfem_sys::GridFunction;

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }
}

impl GridFunction {
    /// Project `coeff` [`Coefficient`] to this [`GridFunction`].
    ///
    /// The projection computation depends on the choice of the [`FiniteElementSpace`] `fespace`.
    ///
    /// Note that this is usually interpolation at the degrees of freedom in each element (not L2 projection).
    pub fn project_coefficient(&mut self, coeff: &mut Coefficient) {
        self.into_pin_mut()
            .ProjectCoefficient5(coeff.into_pin_mut());
    }

    pub fn save_to_file(&self, path: &str, precision: i32) {
        let_cxx_string!(fname = path);
        unsafe {
            self.into_ref()
                .Save1(fname.as_ptr().cast(), c_int(precision));
        }
    }

    pub fn get_own_fec(&self) -> Option<&FiniteElementCollection> {
        let ptr = mfem_sys::GridFunction_OwnFEC(self.into_ref());
        if !ptr.is_null() {
            Some(FiniteElementCollection::from_ref(unsafe { &*ptr }))
        } else {
            None
        }
    }
}

impl Deref for GridFunction {
    type Target = Vector;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for GridFunction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

////////////////
// LinearForm //
////////////////

pub struct OwnedLinearForm {
    inner: UniquePtr<mfem_sys::LinearForm>,
}

impl OwnedLinearForm {
    pub fn new(fespace: &FiniteElementSpace) -> Self {
        let inner = UniquePtr::emplace(unsafe { mfem_sys::LinearForm::new1(fespace.inner) });
        Self { inner }
    }
}

pub struct LinearForm {
    inner: *mut mfem_sys::LinearForm,
}

impl ThinWrapper for LinearForm {
    type Inner = mfem_sys::LinearForm;

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }
}

impl LinearForm {
    pub fn add_domain_integrator(&mut self, lfi: OwnedLinearFormIntegrator) {
        unsafe {
            self.into_pin_mut()
                .AddDomainIntegrator(lfi.inner.into_raw());
        }
    }

    pub fn assemble(&mut self) {
        self.into_pin_mut().Assemble();
    }
}

// impl<'fes> VectorLike for LinearForm<'fes> {}

// impl<'fes> AsBase<mfem_sys::Vector> for LinearForm<'fes> {
//     fn as_base(&self) -> &mfem_sys::Vector {
//         mfem_sys::LinearForm_as_Vector(&self.inner)
//     }
// }

// impl<'fes> AsBaseMut<mfem_sys::Vector> for LinearForm<'fes> {
//     fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::Vector> {
//         mfem_sys::LinearForm_as_mut_Vector(self.inner.pin_mut())
//     }
// }

/////////////////
// Coefficient //
/////////////////

pub struct OwnedCoefficient {
    inner: UniquePtr<mfem_sys::Coefficient>,
}

pub struct Coefficient {
    inner: *mut mfem_sys::Coefficient,
}

impl ThinWrapper for Coefficient {
    type Inner = mfem_sys::Coefficient;

    fn into_ref(&self) -> &Self::Inner {
        unsafe { std::mem::transmute(self) }
    }

    fn into_pin_mut(&mut self) -> Pin<&mut Self::Inner> {
        unsafe { std::mem::transmute(self) }
    }

    fn from_ref(r: &Self::Inner) -> &Self {
        unsafe { std::mem::transmute(r) }
    }

    fn from_pin_mut(r: Pin<&mut Self::Inner>) -> &mut Self {
        unsafe { std::mem::transmute(r) }
    }
}

// /////////////////////////
// // ConstantCoefficient //
// /////////////////////////

// pub struct ConstantCoefficient {
//     inner: UniquePtr<mfem_sys::ConstantCoefficient>,
// }

// impl ConstantCoefficient {
//     pub fn new(value: f64) -> Self {
//         let inner = mfem_sys::ConstantCoefficient_ctor(value);
//         Self { inner }
//     }
// }

// impl Coefficient for ConstantCoefficient {}

// impl AsBase<mfem_sys::Coefficient> for ConstantCoefficient {
//     fn as_base(&self) -> &mfem_sys::Coefficient {
//         mfem_sys::ConstantCoefficient_as_Coeff(&self.inner)
//     }
// }

//////////////////////////
// LinearFormIntegrator //
//////////////////////////

pub struct OwnedLinearFormIntegrator {
    inner: UniquePtr<mfem_sys::LinearFormIntegrator>,
}

pub struct LinearFormIntegrator {
    inner: *mut mfem_sys::LinearFormIntegrator,
}

// ////////////////////////
// // DomainLFIntegrator //
// ////////////////////////

// pub struct DomainLFIntegrator<'coeff> {
//     inner: UniquePtr<mfem_sys::DomainLFIntegrator<'coeff>>,
// }

// impl<'coeff> DomainLFIntegrator<'coeff> {
//     pub fn new(coeff: &'coeff dyn Coefficient, a: i32, b: i32) -> Self {
//         let inner = mfem_sys::DomainLFIntegrator_ctor_ab(coeff.as_base(), a, b);
//         Self { inner }
//     }
// }

// impl<'coeff> LinearFormIntegrator for DomainLFIntegrator<'coeff> {}

// impl<'coeff> AsBase<mfem_sys::LinearFormIntegrator> for DomainLFIntegrator<'coeff> {
//     fn as_base(&self) -> &mfem_sys::LinearFormIntegrator {
//         mfem_sys::DomainLFIntegrator_as_LFI(&self.inner)
//     }
// }

// impl<'coeff> IntoBase<UniquePtr<mfem_sys::LinearFormIntegrator>>
//     for DomainLFIntegrator<'coeff>
// {
//     fn into_base(self) -> UniquePtr<mfem_sys::LinearFormIntegrator> {
//         mfem_sys::DomainLFIntegrator_into_LFI(self.inner)
//     }
// }

// //////////////////
// // BilinearForm //
// //////////////////

// pub struct BilinearForm<'fes> {
//     inner: UniquePtr<mfem_sys::BilinearForm<'fes>>,
// }

// impl<'fes> BilinearForm<'fes> {
//     pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
//         let inner = mfem_sys::BilinearForm_ctor_fes(&fespace.inner);
//         Self { inner }
//     }

//     pub fn add_domain_integrator<Bfi>(&mut self, bfi: Bfi)
//     where
//         Bfi: BilinearFormIntegrator,
//     {
//         mfem_sys::BilinearForm_AddDomainIntegrator(self.inner.pin_mut(), bfi.into_base());
//     }

//     pub fn assemble(&mut self, skip_zeros: bool) {
//         self.inner
//             .pin_mut()
//             .Assemble(if skip_zeros { 1 } else { 0 })
//     }

//     pub fn form_linear_system<X, B>(
//         &self,
//         ess_tdof_list: &ArrayInt,
//         x: &X,
//         b: &B,
//         a_mat: &mut OperatorHandle,
//         x_vec: &mut Vector,
//         b_vec: &mut Vector,
//     ) where
//         X: VectorLike,
//         B: VectorLike,
//     {
//         mfem_sys::BilinearForm_FormLinearSystem(
//             &self.inner,
//             &ess_tdof_list.inner,
//             &x.as_base(),
//             &b.as_base(),
//             a_mat.inner.pin_mut(),
//             x_vec.inner.pin_mut(),
//             b_vec.inner.pin_mut(),
//         );
//     }

//     pub fn recover_fem_solution<B, X>(&mut self, x_vec: &Vector, b_vec: &B, x: &mut X)
//     where
//         B: VectorLike,
//         X: VectorLike,
//     {
//         self.inner
//             .pin_mut()
//             .RecoverFEMSolution(&x_vec.inner, &b_vec.as_base(), x.as_base_mut());
//     }
// }

// ////////////////////////////
// // BilinearFormIntegrator //
// ////////////////////////////

// pub trait BilinearFormIntegrator:
//     AsBase<mfem_sys::BilinearFormIntegrator>
//     + IntoBase<UniquePtr<mfem_sys::BilinearFormIntegrator>>
// {
//     // TODO(mkovaxx)
// }

// /////////////////////////
// // DiffusionIntegrator //
// /////////////////////////

// pub struct DiffusionIntegrator<'coeff> {
//     inner: UniquePtr<mfem_sys::DiffusionIntegrator<'coeff>>,
// }

// impl<'coeff> DiffusionIntegrator<'coeff> {
//     pub fn new(coeff: &'coeff dyn Coefficient) -> Self {
//         let inner = mfem_sys::DiffusionIntegrator_ctor(coeff.as_base());
//         Self { inner }
//     }
// }

// impl<'coeff> BilinearFormIntegrator for DiffusionIntegrator<'coeff> {}

// impl<'coeff> AsBase<mfem_sys::BilinearFormIntegrator> for DiffusionIntegrator<'coeff> {
//     fn as_base(&self) -> &mfem_sys::BilinearFormIntegrator {
//         mfem_sys::DiffusionIntegrator_as_BFI(&self.inner)
//     }
// }

// impl<'coeff> IntoBase<UniquePtr<mfem_sys::BilinearFormIntegrator>>
//     for DiffusionIntegrator<'coeff>
// {
//     fn into_base(self) -> UniquePtr<mfem_sys::BilinearFormIntegrator> {
//         mfem_sys::DiffusionIntegrator_into_BFI(self.inner)
//     }
// }

// //////////////
// // Operator //
// //////////////

// pub trait Operator: AsBase<mfem_sys::Operator> {
//     fn height(&self) -> i32 {
//         self.as_base().Height()
//     }
// }

// ////////////////////
// // OperatorHandle //
// ////////////////////

// pub use mfem_sys::Operator_Type as OperatorType;

// pub struct OperatorHandle {
//     inner: UniquePtr<mfem_sys::OperatorHandle>,
// }

// impl OperatorHandle {
//     pub fn new() -> Self {
//         let inner = mfem_sys::OperatorHandle_ctor();
//         Self { inner }
//     }

//     pub fn get_type(&self) -> OperatorType {
//         self.inner.Type()
//     }
// }

// impl Operator for OperatorHandle {}

// impl AsBase<mfem_sys::Operator> for OperatorHandle {
//     fn as_base(&self) -> &mfem_sys::Operator {
//         mfem_sys::OperatorHandle_as_ref(&self.inner)
//     }
// }

// //////////////////
// // SparseMatrix //
// //////////////////

// pub struct SparseMatrix {
//     inner: UniquePtr<mfem_sys::SparseMatrix>,
// }

// impl<'a> TryFrom<OperatorHandle> for SparseMatrix {
//     type Error = MfemError;

//     fn try_from(value: OperatorHandle) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

// pub struct SparseMatrixRef<'a> {
//     inner: &'a mfem_sys::SparseMatrix,
// }

// impl<'a> TryFrom<&'a OperatorHandle> for SparseMatrixRef<'a> {
//     // TODO(mkovaxx)
//     type Error = MfemError;

//     fn try_from(value: &'a OperatorHandle) -> Result<Self, Self::Error> {
//         let inner =
//             mfem_sys::OperatorHandle_try_as_SparseMatrix(&value.inner).map_err(|_| {
//                 MfemError::OperatorHandleTypeMismatch(
//                     OperatorType::MFEM_SPARSEMAT,
//                     value.get_type(),
//                 )
//             })?;
//         Ok(Self { inner })
//     }
// }

// ////////////
// // Solver //
// ////////////

// pub trait Solver: AsBaseMut<mfem_sys::Solver> {
//     // TODO(mkovaxx)
// }

// ////////////////
// // GSSmoother //
// ////////////////

// pub struct GsSmoother<'mat> {
//     inner: UniquePtr<mfem_sys::GSSmoother<'mat>>,
// }

// impl<'mat> GsSmoother<'mat> {
//     pub fn new(a: &SparseMatrixRef<'mat>, t: i32, it: i32) -> Self {
//         let inner = mfem_sys::GSSmoother_ctor(a.inner, t, it);
//         Self { inner }
//     }
// }

// impl<'mat> Solver for GsSmoother<'mat> {}

// impl<'mat> AsBaseMut<mfem_sys::Solver> for GsSmoother<'mat> {
//     fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::Solver> {
//         mfem_sys::GSSmoother_as_mut_Solver(self.inner.pin_mut())
//     }
// }

// /////////
// // PCG //
// /////////

// pub fn solve_with_pcg<Op, So>(
//     a_mat: &Op,
//     solver: &mut So,
//     b_vec: &Vector,
//     x_vec: &mut Vector,
//     print_iter: i32,
//     max_num_iter: i32,
//     rtolerance: f64,
//     atolerance: f64,
// ) where
//     Op: Operator,
//     So: Solver,
// {
//     mfem_sys::PCG(
//         a_mat.as_base(),
//         solver.as_base_mut(),
//         &b_vec.inner,
//         x_vec.inner.pin_mut(),
//         print_iter,
//         max_num_iter,
//         rtolerance,
//         atolerance,
//     );
// }

// ///////////
// // Error //
// ///////////

// #[derive(Error, Debug)]
// pub enum MfemError {
//     #[error("OperatorHandle type mismatch: expected {0:?} got {1:?}")]
//     OperatorHandleTypeMismatch(OperatorType, OperatorType),
// }
