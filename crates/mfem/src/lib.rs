use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;
use std::ptr::null;

use autocxx::c_int;
use autocxx::prelude::Emplace;
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

#[repr(transparent)]
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

#[repr(transparent)]
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
    pub fn set_all(&mut self, value: f64) {
        mfem_sys::Vector_set_all(self.into_pin_mut(), value);
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

#[repr(transparent)]
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

#[repr(transparent)]
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

#[repr(transparent)]
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

#[repr(transparent)]
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
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for H1FeCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

////////////////////////
// FiniteElementSpace //
////////////////////////

pub use mfem_sys::Ordering_Type as OrderingType;

#[repr(transparent)]
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

#[repr(transparent)]
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

#[repr(transparent)]
pub struct OwnedGridFunction {
    inner: UniquePtr<mfem_sys::GridFunction>,
}

impl OwnedGridFunction {
    pub fn new(fespace: &FiniteElementSpace) -> Self {
        let inner = UniquePtr::emplace(unsafe { mfem_sys::GridFunction::new2(fespace.inner) });
        Self { inner }
    }
}

impl Deref for OwnedGridFunction {
    type Target = GridFunction;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedGridFunction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
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
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for GridFunction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

////////////////
// LinearForm //
////////////////

#[repr(transparent)]
pub struct OwnedLinearForm {
    inner: UniquePtr<mfem_sys::LinearForm>,
}

impl OwnedLinearForm {
    pub fn new(fespace: &FiniteElementSpace) -> Self {
        let inner = UniquePtr::emplace(unsafe { mfem_sys::LinearForm::new1(fespace.inner) });
        Self { inner }
    }
}

impl Deref for OwnedLinearForm {
    type Target = LinearForm;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedLinearForm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
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
    pub fn add_domain_integrator<LFI>(&mut self, lfi: LFI)
    where
        LFI: Into<OwnedLinearFormIntegrator>,
    {
        unsafe {
            self.into_pin_mut()
                .AddDomainIntegrator(lfi.into().inner.into_raw());
        }
    }

    pub fn assemble(&mut self) {
        self.into_pin_mut().Assemble();
    }
}

impl Deref for LinearForm {
    type Target = Vector;

    fn deref(&self) -> &Self::Target {
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for LinearForm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

/////////////////
// Coefficient //
/////////////////

#[repr(transparent)]
pub struct OwnedCoefficient {
    inner: UniquePtr<mfem_sys::Coefficient>,
}

#[repr(transparent)]
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

/////////////////////////
// ConstantCoefficient //
/////////////////////////

#[repr(transparent)]
pub struct OwnedConstantCoefficient {
    inner: UniquePtr<mfem_sys::ConstantCoefficient>,
}

impl OwnedConstantCoefficient {
    pub fn new(value: f64) -> Self {
        let inner = UniquePtr::emplace(mfem_sys::ConstantCoefficient::new(value));
        Self { inner }
    }
}

impl Deref for OwnedConstantCoefficient {
    type Target = ConstantCoefficient;

    fn deref(&self) -> &Self::Target {
        ConstantCoefficient::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedConstantCoefficient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        ConstantCoefficient::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct ConstantCoefficient {
    inner: *mut mfem_sys::ConstantCoefficient,
}

impl ThinWrapper for ConstantCoefficient {
    type Inner = mfem_sys::ConstantCoefficient;

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

impl Deref for ConstantCoefficient {
    type Target = Coefficient;

    fn deref(&self) -> &Self::Target {
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for ConstantCoefficient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

//////////////////////////
// LinearFormIntegrator //
//////////////////////////

#[repr(transparent)]
pub struct OwnedLinearFormIntegrator {
    inner: UniquePtr<mfem_sys::LinearFormIntegrator>,
}

impl Deref for OwnedLinearFormIntegrator {
    type Target = LinearFormIntegrator;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedLinearFormIntegrator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct LinearFormIntegrator {
    inner: *mut mfem_sys::LinearFormIntegrator,
}

impl ThinWrapper for LinearFormIntegrator {
    type Inner = mfem_sys::LinearFormIntegrator;

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

////////////////////////
// DomainLFIntegrator //
////////////////////////

#[repr(transparent)]
pub struct OwnedDomainLFIntegrator {
    inner: UniquePtr<mfem_sys::DomainLFIntegrator>,
}

impl Into<OwnedLinearFormIntegrator> for OwnedDomainLFIntegrator {
    fn into(self) -> OwnedLinearFormIntegrator {
        // FIXME?
        unsafe { std::mem::transmute(self) }
    }
}

impl OwnedDomainLFIntegrator {
    pub fn new(coeff: &mut Coefficient, a: i32, b: i32) -> Self {
        let inner = UniquePtr::emplace(mfem_sys::DomainLFIntegrator::new(
            coeff.into_pin_mut(),
            c_int(a),
            c_int(b),
        ));
        Self { inner }
    }
}

impl Deref for OwnedDomainLFIntegrator {
    type Target = DomainLFIntegrator;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedDomainLFIntegrator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct DomainLFIntegrator {
    inner: *mut mfem_sys::DomainLFIntegrator,
}

impl ThinWrapper for DomainLFIntegrator {
    type Inner = mfem_sys::DomainLFIntegrator;

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

//////////////////
// BilinearForm //
//////////////////

#[repr(transparent)]
pub struct OwnedBilinearForm {
    inner: UniquePtr<mfem_sys::BilinearForm>,
}

impl OwnedBilinearForm {
    pub fn new(fespace: &FiniteElementSpace) -> Self {
        let inner = UniquePtr::emplace(unsafe { mfem_sys::BilinearForm::new2(fespace.inner) });
        Self { inner }
    }
}

impl Deref for OwnedBilinearForm {
    type Target = BilinearForm;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedBilinearForm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct BilinearForm {
    inner: *mut mfem_sys::BilinearForm,
}

impl ThinWrapper for BilinearForm {
    type Inner = mfem_sys::BilinearForm;

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

impl BilinearForm {
    pub fn add_domain_integrator<Bfi>(&mut self, bfi: Bfi)
    where
        Bfi: Into<OwnedBilinearFormIntegrator>,
    {
        unsafe {
            self.into_pin_mut()
                .AddDomainIntegrator(bfi.into().inner.into_raw());
        }
    }

    pub fn assemble(&mut self, skip_zeros: bool) {
        self.into_pin_mut().Assemble(c_int(skip_zeros as i32))
    }

    pub fn form_linear_system(
        &mut self,
        ess_tdof_list: &ArrayInt,
        x: &mut Vector,
        b: &mut Vector,
        a_mat: &mut OperatorHandle,
        x_vec: &mut Vector,
        b_vec: &mut Vector,
    ) {
        let copy_interior = false;
        mfem_sys::BilinearForm::FormLinearSystem(
            self.into_pin_mut(),
            ess_tdof_list.into_ref(),
            x.into_pin_mut(),
            b.into_pin_mut(),
            a_mat.into_pin_mut(),
            x_vec.into_pin_mut(),
            b_vec.into_pin_mut(),
            c_int(copy_interior as i32),
        );
    }

    pub fn recover_fem_solution(&mut self, x_vec: &Vector, b_vec: &Vector, x: &mut Vector) {
        self.into_pin_mut().RecoverFEMSolution(
            x_vec.into_ref(),
            b_vec.into_ref(),
            x.into_pin_mut(),
        );
    }
}

////////////////////////////
// BilinearFormIntegrator //
////////////////////////////

#[repr(transparent)]
pub struct OwnedBilinearFormIntegrator {
    inner: UniquePtr<mfem_sys::BilinearFormIntegrator>,
}

/////////////////////////
// DiffusionIntegrator //
/////////////////////////

#[repr(transparent)]
pub struct OwnedDiffusionIntegrator {
    inner: UniquePtr<mfem_sys::DiffusionIntegrator>,
}

impl OwnedDiffusionIntegrator {
    pub fn new(coeff: &mut Coefficient) -> Self {
        let integration_rule: *const mfem_sys::IntegrationRule = null();
        let inner = UniquePtr::emplace(unsafe {
            mfem_sys::DiffusionIntegrator::new1(coeff.into_pin_mut(), integration_rule)
        });
        Self { inner }
    }
}

impl Into<OwnedBilinearFormIntegrator> for OwnedDiffusionIntegrator {
    fn into(self) -> OwnedBilinearFormIntegrator {
        // FIXME?
        unsafe { std::mem::transmute(self) }
    }
}

impl Deref for OwnedDiffusionIntegrator {
    type Target = DiffusionIntegrator;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

#[repr(transparent)]
pub struct DiffusionIntegrator {
    inner: *mut mfem_sys::DiffusionIntegrator,
}

impl ThinWrapper for DiffusionIntegrator {
    type Inner = mfem_sys::DiffusionIntegrator;

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

//////////////
// Operator //
//////////////

#[repr(transparent)]
pub struct OwnedOperator {
    inner: UniquePtr<mfem_sys::Operator>,
}

#[repr(transparent)]
pub struct Operator {
    inner: *mut mfem_sys::Operator,
}

impl ThinWrapper for Operator {
    type Inner = mfem_sys::Operator;

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

impl Operator {
    pub fn width(&self) -> usize {
        self.into_ref().Width() as usize
    }

    pub fn height(&self) -> usize {
        self.into_ref().Height() as usize
    }

    pub fn get_type(&self) -> OperatorType {
        self.into_ref().GetType()
    }
}

////////////////////
// OperatorHandle //
////////////////////

pub use mfem_sys::Operator_Type as OperatorType;

#[repr(transparent)]
pub struct OwnedOperatorHandle {
    inner: UniquePtr<mfem_sys::OperatorHandle>,
}

impl OwnedOperatorHandle {
    pub fn new() -> Self {
        let inner = UniquePtr::emplace(mfem_sys::OperatorHandle::new());
        Self { inner }
    }
}

impl Into<OwnedOperator> for OwnedOperatorHandle {
    fn into(self) -> OwnedOperator {
        // FIXME?
        unsafe { std::mem::transmute(self) }
    }
}

impl Deref for OwnedOperatorHandle {
    type Target = OperatorHandle;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedOperatorHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct OperatorHandle {
    inner: *mut mfem_sys::OperatorHandle,
}

impl ThinWrapper for OperatorHandle {
    type Inner = mfem_sys::OperatorHandle;

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

impl Deref for OperatorHandle {
    type Target = Operator;

    fn deref(&self) -> &Self::Target {
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for OperatorHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // FIXME: use helper function from mfem_sys
        unsafe { std::mem::transmute(self) }
    }
}

//////////////////
// SparseMatrix //
//////////////////

#[repr(transparent)]
pub struct OwnedSparseMatrix {
    inner: UniquePtr<mfem_sys::SparseMatrix>,
}

impl Deref for OwnedSparseMatrix {
    type Target = SparseMatrix;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedSparseMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct SparseMatrix {
    inner: *mut mfem_sys::SparseMatrix,
}

impl ThinWrapper for SparseMatrix {
    type Inner = mfem_sys::SparseMatrix;

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

impl TryFrom<&Operator> for SparseMatrix {
    // TODO(mkovaxx)
    type Error = MfemError;

    fn try_from(value: &Operator) -> Result<Self, Self::Error> {
        if value.get_type() == OperatorType::MFEM_SPARSEMAT {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(MfemError::OperatorHandleTypeMismatch(
                OperatorType::MFEM_SPARSEMAT,
                value.get_type(),
            ))
        }
    }
}

////////////
// Solver //
////////////

#[repr(transparent)]
pub struct OwnedSolver {
    inner: UniquePtr<mfem_sys::Solver>,
}

impl Deref for OwnedSolver {
    type Target = Solver;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedSolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct Solver {
    inner: *mut mfem_sys::Solver,
}

impl ThinWrapper for Solver {
    type Inner = mfem_sys::Solver;

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

////////////////
// GsSmoother //
////////////////

#[repr(transparent)]
pub struct OwnedGsSmoother {
    inner: UniquePtr<mfem_sys::GSSmoother>,
}

impl OwnedGsSmoother {
    pub fn new(a: &SparseMatrix, t: i32, it: i32) -> Self {
        let inner = UniquePtr::emplace(mfem_sys::GSSmoother::new1(
            a.into_ref(),
            c_int(t),
            c_int(it),
        ));
        Self { inner }
    }
}

impl Into<OwnedSolver> for OwnedGsSmoother {
    fn into(self) -> OwnedSolver {
        // FIXME?
        unsafe { std::mem::transmute(self) }
    }
}

impl Deref for OwnedGsSmoother {
    type Target = GsSmoother;

    fn deref(&self) -> &Self::Target {
        Self::Target::from_ref(&self.inner)
    }
}

impl DerefMut for OwnedGsSmoother {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::Target::from_pin_mut(self.inner.pin_mut())
    }
}

#[repr(transparent)]
pub struct GsSmoother {
    inner: *mut mfem_sys::GSSmoother,
}

impl ThinWrapper for GsSmoother {
    type Inner = mfem_sys::GSSmoother;

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

impl Deref for GsSmoother {
    type Target = Solver;

    fn deref(&self) -> &Self::Target {
        // FIXME?
        unsafe { std::mem::transmute(self) }
    }
}

impl DerefMut for GsSmoother {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // FIXME?
        unsafe { std::mem::transmute(self) }
    }
}

/////////
// PCG //
/////////

pub fn solve_with_pcg(
    a_mat: &Operator,
    solver: &mut Solver,
    b_vec: &Vector,
    x_vec: &mut Vector,
    print_iter: bool,
    max_num_iter: i32,
    rtolerance: f64,
    atolerance: f64,
) {
    mfem_sys::PCG(
        a_mat.into_ref(),
        solver.into_pin_mut(),
        b_vec.into_ref(),
        x_vec.into_pin_mut(),
        print_iter as i32,
        max_num_iter,
        rtolerance.into(),
        atolerance.into(),
    );
}

///////////
// Error //
///////////

#[derive(Error, Debug)]
pub enum MfemError {
    #[error("OperatorHandle type mismatch: expected {0:?} got {1:?}")]
    OperatorHandleTypeMismatch(OperatorType, OperatorType),
}
