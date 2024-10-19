use cxx::memory::UniquePtrTarget;
use cxx::{let_cxx_string, UniquePtr};
use thiserror::Error;

trait AsBase<T> {
    fn as_base(&self) -> &T;
}

trait AsBaseMut<T> {
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut T>;
}

trait IntoBase<T> {
    fn into_base(self) -> T;
}

// Every type T is also its own base type
impl<T> AsBase<T> for T {
    fn as_base(&self) -> &T {
        self
    }
}

// Every type T is also its own base type
impl<T> AsBaseMut<T> for UniquePtr<T>
where
    T: UniquePtrTarget,
{
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut T> {
        self.pin_mut()
    }
}

// Every type T is also its own base type
impl<T> IntoBase<UniquePtr<T>> for UniquePtr<T>
where
    T: UniquePtrTarget,
{
    fn into_base(self) -> UniquePtr<T> {
        self
    }
}

//////////////
// ArrayInt //
//////////////

pub struct ArrayInt {
    inner: UniquePtr<mfem_sys::ffi::ArrayInt>,
}

pub struct ArrayIntRef<'a> {
    inner: &'a mfem_sys::ffi::ArrayInt,
}

impl ArrayInt {
    pub fn new() -> Self {
        let inner = mfem_sys::ffi::ArrayInt_ctor();
        Self { inner }
    }

    pub fn with_len(len: usize) -> Self {
        let inner = mfem_sys::ffi::ArrayInt_ctor_size(len as i32);
        Self { inner }
    }

    pub fn set_all(&mut self, value: i32) {
        mfem_sys::ffi::ArrayInt_SetAll(self.inner.pin_mut(), value);
    }
}

impl<'a> ArrayIntRef<'a> {
    pub fn as_slice(&self) -> &[i32] {
        let data = self.inner.GetData();
        let size = self.inner.Size() as usize;
        unsafe { std::slice::from_raw_parts(data, size) }
    }

    pub fn iter(&self) -> impl Iterator<Item = &i32> {
        self.as_slice().iter()
    }
}

////////////////
// VectorLike //
////////////////

pub trait VectorLike: AsBase<mfem_sys::ffi::Vector> + AsBaseMut<mfem_sys::ffi::Vector> {
    // TODO(mkovaxx)
}

////////////
// Vector //
////////////

pub struct Vector {
    inner: UniquePtr<mfem_sys::ffi::Vector>,
}

impl Vector {
    pub fn new() -> Self {
        let inner = mfem_sys::ffi::Vector_ctor();
        Self { inner }
    }
}

impl VectorLike for Vector {}

impl AsBase<mfem_sys::ffi::Vector> for Vector {
    fn as_base(&self) -> &mfem_sys::ffi::Vector {
        &self.inner
    }
}

impl AsBaseMut<mfem_sys::ffi::Vector> for Vector {
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::ffi::Vector> {
        self.inner.pin_mut()
    }
}

//////////
// Mesh //
//////////

pub struct Mesh {
    inner: UniquePtr<mfem_sys::ffi::Mesh>,
}

impl Mesh {
    pub fn new() -> Self {
        let inner = mfem_sys::ffi::Mesh_ctor();
        Self { inner }
    }

    pub fn from_file(path: &str) -> Result<Self, MfemError> {
        let generate_edges = 1;
        let refine = 1;
        let fix_orientation = true;
        let_cxx_string!(mesh_path = path);
        let inner =
            mfem_sys::ffi::Mesh_ctor_file(&mesh_path, generate_edges, refine, fix_orientation);
        Ok(Self { inner })
    }

    pub fn dimension(&self) -> i32 {
        self.inner.Dimension()
    }

    pub fn get_num_elems(&self) -> i32 {
        self.inner.GetNE()
    }

    pub fn get_nodes<'fes, 'a: 'fes>(&'a self) -> Option<GridFunctionRef<'fes, 'a>> {
        mfem_sys::ffi::Mesh_GetNodes(&self.inner)
            .ok()
            .map(|grid_func| GridFunctionRef { inner: grid_func })
    }

    pub fn get_bdr_attributes<'a>(&'a self) -> ArrayIntRef<'a> {
        let inner = mfem_sys::ffi::Mesh_bdr_attributes(&self.inner);
        ArrayIntRef { inner }
    }

    pub fn uniform_refinement(&mut self, ref_algo: RefAlgo) {
        self.inner.pin_mut().UniformRefinement(ref_algo as i32);
    }

    pub fn save_to_file(&self, path: &str, precision: i32) {
        let_cxx_string!(fname = path);
        self.inner.Save(&fname, precision);
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

pub use mfem_sys::ffi::BasisType;

/////////////////////////////
// FiniteElementCollection //
/////////////////////////////

pub trait FiniteElementCollection: AsBase<mfem_sys::ffi::FiniteElementCollection> {
    fn get_name(&self) -> String {
        let ptr = self.as_base().Name();
        assert!(!ptr.is_null());
        let name = unsafe { std::ffi::CStr::from_ptr(ptr) };
        name.to_owned().into_string().expect("Valid string")
    }
}

impl FiniteElementCollection for mfem_sys::ffi::FiniteElementCollection {}

/////////////////////
// H1_FECollection //
/////////////////////

pub struct H1FeCollection {
    inner: UniquePtr<mfem_sys::ffi::H1_FECollection>,
}

impl H1FeCollection {
    pub fn new(p: i32, dim: i32, btype: BasisType) -> Self {
        let inner = mfem_sys::ffi::H1_FECollection_ctor(p, dim, btype.repr);
        Self { inner }
    }
}

impl FiniteElementCollection for H1FeCollection {}

impl AsBase<mfem_sys::ffi::FiniteElementCollection> for H1FeCollection {
    fn as_base(&self) -> &mfem_sys::ffi::FiniteElementCollection {
        mfem_sys::ffi::H1_FECollection_as_FEC(&self.inner)
    }
}

////////////////////////
// FiniteElementSpace //
////////////////////////

pub use mfem_sys::ffi::OrderingType;

pub struct FiniteElementSpace<'mesh, 'fec> {
    inner: UniquePtr<mfem_sys::ffi::FiniteElementSpace<'mesh, 'fec>>,
}

impl<'mesh, 'fec> FiniteElementSpace<'mesh, 'fec> {
    pub fn new(
        mesh: &'mesh Mesh,
        fec: &'fec dyn FiniteElementCollection,
        vdim: i32,
        ordering: OrderingType,
    ) -> Self {
        let inner =
            mfem_sys::ffi::FiniteElementSpace_ctor(&mesh.inner, &fec.as_base(), vdim, ordering);
        Self { inner }
    }

    pub fn get_true_vsize(&self) -> i32 {
        self.inner.GetTrueVSize()
    }

    pub fn get_essential_true_dofs(
        &self,
        bdr_attr_is_ess: &ArrayInt,
        ess_tdof_list: &mut ArrayInt,
        component: Option<usize>,
    ) {
        mfem_sys::ffi::FiniteElementSpace_GetEssentialTrueDofs(
            &self.inner,
            &bdr_attr_is_ess.inner,
            ess_tdof_list.inner.pin_mut(),
            component.map(|c| c as i32).unwrap_or(-1),
        );
    }
}

//////////////////
// GridFunction //
//////////////////

pub struct GridFunction<'fes> {
    inner: UniquePtr<mfem_sys::ffi::GridFunction<'fes>>,
}

pub struct GridFunctionRef<'fes, 'a> {
    inner: &'a mfem_sys::ffi::GridFunction<'fes>,
}

impl<'fes> GridFunction<'fes> {
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        let inner = mfem_sys::ffi::GridFunction_ctor_fes(&fespace.inner);
        Self { inner }
    }

    /// Project `coeff` [`Coefficient`] to this [`GridFunction`].
    ///
    /// The projection computation depends on the choice of the [`FiniteElementSpace`] `fespace`.
    ///
    /// Note that this is usually interpolation at the degrees of freedom in each element (not L2 projection).
    pub fn project_coefficient(&mut self, coeff: &dyn Coefficient) {
        mfem_sys::ffi::GridFunction_ProjectCoefficient(self.inner.pin_mut(), coeff.as_base());
    }

    pub fn set_all(&mut self, value: f64) {
        mfem_sys::ffi::GridFunction_SetAll(self.inner.pin_mut(), value);
    }

    pub fn save_to_file(&self, path: &str, precision: i32) {
        let_cxx_string!(fname = path);
        mfem_sys::ffi::GridFunction_Save(&self.inner, &fname, precision);
    }
}

impl<'fes, 'a> GridFunctionRef<'fes, 'a> {
    pub fn get_own_fec(&self) -> Option<&dyn FiniteElementCollection> {
        mfem_sys::ffi::GridFunction_OwnFEC(self.inner)
            .ok()
            .map(|fec| fec as &dyn FiniteElementCollection)
    }
}

impl<'fes> VectorLike for GridFunction<'fes> {}

impl<'fes> AsBase<mfem_sys::ffi::Vector> for GridFunction<'fes> {
    fn as_base(&self) -> &mfem_sys::ffi::Vector {
        mfem_sys::ffi::GridFunction_as_Vector(&self.inner)
    }
}

impl<'fes> AsBaseMut<mfem_sys::ffi::Vector> for GridFunction<'fes> {
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::ffi::Vector> {
        mfem_sys::ffi::GridFunction_as_mut_Vector(self.inner.pin_mut())
    }
}

////////////////
// LinearForm //
////////////////

pub struct LinearForm<'fes> {
    inner: UniquePtr<mfem_sys::ffi::LinearForm<'fes>>,
}

impl<'fes> LinearForm<'fes> {
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        let inner = mfem_sys::ffi::LinearForm_ctor_fes(&fespace.inner);
        Self { inner }
    }

    pub fn add_domain_integrator<Lfi>(&mut self, lfi: Lfi)
    where
        Lfi: LinearFormIntegrator,
    {
        mfem_sys::ffi::LinearForm_AddDomainIntegrator(self.inner.pin_mut(), lfi.into_base());
    }

    pub fn assemble(&mut self) {
        self.inner.pin_mut().Assemble();
    }
}

impl<'fes> VectorLike for LinearForm<'fes> {}

impl<'fes> AsBase<mfem_sys::ffi::Vector> for LinearForm<'fes> {
    fn as_base(&self) -> &mfem_sys::ffi::Vector {
        mfem_sys::ffi::LinearForm_as_Vector(&self.inner)
    }
}

impl<'fes> AsBaseMut<mfem_sys::ffi::Vector> for LinearForm<'fes> {
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::ffi::Vector> {
        mfem_sys::ffi::LinearForm_as_mut_Vector(self.inner.pin_mut())
    }
}

/////////////////
// Coefficient //
/////////////////

pub trait Coefficient: AsBase<mfem_sys::ffi::Coefficient> {
    // TODO(mkovaxx)
}

/////////////////////////
// ConstantCoefficient //
/////////////////////////

pub struct ConstantCoefficient {
    inner: UniquePtr<mfem_sys::ffi::ConstantCoefficient>,
}

impl ConstantCoefficient {
    pub fn new(value: f64) -> Self {
        let inner = mfem_sys::ffi::ConstantCoefficient_ctor(value);
        Self { inner }
    }
}

impl Coefficient for ConstantCoefficient {}

impl AsBase<mfem_sys::ffi::Coefficient> for ConstantCoefficient {
    fn as_base(&self) -> &mfem_sys::ffi::Coefficient {
        mfem_sys::ffi::ConstantCoefficient_as_Coeff(&self.inner)
    }
}

//////////////////////////
// LinearFormIntegrator //
//////////////////////////

pub trait LinearFormIntegrator:
    AsBase<mfem_sys::ffi::LinearFormIntegrator>
    + IntoBase<UniquePtr<mfem_sys::ffi::LinearFormIntegrator>>
{
    // TODO(mkovaxx)
}

////////////////////////
// DomainLFIntegrator //
////////////////////////

pub struct DomainLFIntegrator<'coeff> {
    inner: UniquePtr<mfem_sys::ffi::DomainLFIntegrator<'coeff>>,
}

impl<'coeff> DomainLFIntegrator<'coeff> {
    pub fn new(coeff: &'coeff dyn Coefficient, a: i32, b: i32) -> Self {
        let inner = mfem_sys::ffi::DomainLFIntegrator_ctor_ab(coeff.as_base(), a, b);
        Self { inner }
    }
}

impl<'coeff> LinearFormIntegrator for DomainLFIntegrator<'coeff> {}

impl<'coeff> AsBase<mfem_sys::ffi::LinearFormIntegrator> for DomainLFIntegrator<'coeff> {
    fn as_base(&self) -> &mfem_sys::ffi::LinearFormIntegrator {
        mfem_sys::ffi::DomainLFIntegrator_as_LFI(&self.inner)
    }
}

impl<'coeff> IntoBase<UniquePtr<mfem_sys::ffi::LinearFormIntegrator>>
    for DomainLFIntegrator<'coeff>
{
    fn into_base(self) -> UniquePtr<mfem_sys::ffi::LinearFormIntegrator> {
        mfem_sys::ffi::DomainLFIntegrator_into_LFI(self.inner)
    }
}

//////////////////
// BilinearForm //
//////////////////

pub struct BilinearForm<'fes> {
    inner: UniquePtr<mfem_sys::ffi::BilinearForm<'fes>>,
}

impl<'fes> BilinearForm<'fes> {
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        let inner = mfem_sys::ffi::BilinearForm_ctor_fes(&fespace.inner);
        Self { inner }
    }

    pub fn add_domain_integrator<Bfi>(&mut self, bfi: Bfi)
    where
        Bfi: BilinearFormIntegrator,
    {
        mfem_sys::ffi::BilinearForm_AddDomainIntegrator(self.inner.pin_mut(), bfi.into_base());
    }

    pub fn assemble(&mut self, skip_zeros: bool) {
        self.inner
            .pin_mut()
            .Assemble(if skip_zeros { 1 } else { 0 })
    }

    pub fn form_linear_system<X, B>(
        &self,
        ess_tdof_list: &ArrayInt,
        x: &X,
        b: &B,
        a_mat: &mut OperatorHandle,
        x_vec: &mut Vector,
        b_vec: &mut Vector,
    ) where
        X: VectorLike,
        B: VectorLike,
    {
        mfem_sys::ffi::BilinearForm_FormLinearSystem(
            &self.inner,
            &ess_tdof_list.inner,
            &x.as_base(),
            &b.as_base(),
            a_mat.inner.pin_mut(),
            x_vec.inner.pin_mut(),
            b_vec.inner.pin_mut(),
        );
    }

    pub fn recover_fem_solution<B, X>(&mut self, x_vec: &Vector, b_vec: &B, x: &mut X)
    where
        B: VectorLike,
        X: VectorLike,
    {
        self.inner
            .pin_mut()
            .RecoverFEMSolution(&x_vec.inner, &b_vec.as_base(), x.as_base_mut());
    }
}

////////////////////////////
// BilinearFormIntegrator //
////////////////////////////

pub trait BilinearFormIntegrator:
    AsBase<mfem_sys::ffi::BilinearFormIntegrator>
    + IntoBase<UniquePtr<mfem_sys::ffi::BilinearFormIntegrator>>
{
    // TODO(mkovaxx)
}

/////////////////////////
// DiffusionIntegrator //
/////////////////////////

pub struct DiffusionIntegrator<'coeff> {
    inner: UniquePtr<mfem_sys::ffi::DiffusionIntegrator<'coeff>>,
}

impl<'coeff> DiffusionIntegrator<'coeff> {
    pub fn new(coeff: &'coeff dyn Coefficient) -> Self {
        let inner = mfem_sys::ffi::DiffusionIntegrator_ctor(coeff.as_base());
        Self { inner }
    }
}

impl<'coeff> BilinearFormIntegrator for DiffusionIntegrator<'coeff> {}

impl<'coeff> AsBase<mfem_sys::ffi::BilinearFormIntegrator> for DiffusionIntegrator<'coeff> {
    fn as_base(&self) -> &mfem_sys::ffi::BilinearFormIntegrator {
        mfem_sys::ffi::DiffusionIntegrator_as_BFI(&self.inner)
    }
}

impl<'coeff> IntoBase<UniquePtr<mfem_sys::ffi::BilinearFormIntegrator>>
    for DiffusionIntegrator<'coeff>
{
    fn into_base(self) -> UniquePtr<mfem_sys::ffi::BilinearFormIntegrator> {
        mfem_sys::ffi::DiffusionIntegrator_into_BFI(self.inner)
    }
}

//////////////
// Operator //
//////////////

pub trait Operator: AsBase<mfem_sys::ffi::Operator> {
    fn height(&self) -> i32 {
        self.as_base().Height()
    }
}

////////////////////
// OperatorHandle //
////////////////////

pub use mfem_sys::ffi::OperatorType;

pub struct OperatorHandle {
    inner: UniquePtr<mfem_sys::ffi::OperatorHandle>,
}

impl OperatorHandle {
    pub fn new() -> Self {
        let inner = mfem_sys::ffi::OperatorHandle_ctor();
        Self { inner }
    }

    pub fn get_type(&self) -> OperatorType {
        self.inner.Type()
    }
}

impl Operator for OperatorHandle {}

impl AsBase<mfem_sys::ffi::Operator> for OperatorHandle {
    fn as_base(&self) -> &mfem_sys::ffi::Operator {
        mfem_sys::ffi::OperatorHandle_as_ref(&self.inner)
    }
}

//////////////////
// SparseMatrix //
//////////////////

pub struct SparseMatrix {
    inner: UniquePtr<mfem_sys::ffi::SparseMatrix>,
}

impl<'a> TryFrom<OperatorHandle> for SparseMatrix {
    type Error = MfemError;

    fn try_from(value: OperatorHandle) -> Result<Self, Self::Error> {
        todo!()
    }
}

pub struct SparseMatrixRef<'a> {
    inner: &'a mfem_sys::ffi::SparseMatrix,
}

impl<'a> TryFrom<&'a OperatorHandle> for SparseMatrixRef<'a> {
    // TODO(mkovaxx)
    type Error = MfemError;

    fn try_from(value: &'a OperatorHandle) -> Result<Self, Self::Error> {
        let inner =
            mfem_sys::ffi::OperatorHandle_try_as_SparseMatrix(&value.inner).map_err(|_| {
                MfemError::OperatorHandleTypeMismatch(
                    OperatorType::MFEM_SPARSEMAT,
                    value.get_type(),
                )
            })?;
        Ok(Self { inner })
    }
}

////////////
// Solver //
////////////

pub trait Solver: AsBaseMut<mfem_sys::ffi::Solver> {
    // TODO(mkovaxx)
}

////////////////
// GSSmoother //
////////////////

pub struct GsSmoother<'mat> {
    inner: UniquePtr<mfem_sys::ffi::GSSmoother<'mat>>,
}

impl<'mat> GsSmoother<'mat> {
    pub fn new(a: &SparseMatrixRef<'mat>, t: i32, it: i32) -> Self {
        let inner = mfem_sys::ffi::GSSmoother_ctor(a.inner, t, it);
        Self { inner }
    }
}

impl<'mat> Solver for GsSmoother<'mat> {}

impl<'mat> AsBaseMut<mfem_sys::ffi::Solver> for GsSmoother<'mat> {
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::ffi::Solver> {
        mfem_sys::ffi::GSSmoother_as_mut_Solver(self.inner.pin_mut())
    }
}

/////////
// PCG //
/////////

pub fn solve_with_pcg<Op, So>(
    a_mat: &Op,
    solver: &mut So,
    b_vec: &Vector,
    x_vec: &mut Vector,
    print_iter: i32,
    max_num_iter: i32,
    rtolerance: f64,
    atolerance: f64,
) where
    Op: Operator,
    So: Solver,
{
    mfem_sys::ffi::PCG(
        a_mat.as_base(),
        solver.as_base_mut(),
        &b_vec.inner,
        x_vec.inner.pin_mut(),
        print_iter,
        max_num_iter,
        rtolerance,
        atolerance,
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
