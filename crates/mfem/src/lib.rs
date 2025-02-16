use cxx::{let_cxx_string, UniquePtr};
use thiserror::Error;

//////////////
// ArrayInt //
//////////////

pub struct ArrayInt {
    inner: UniquePtr<mfem_sys::ArrayInt>,
}

pub struct ArrayIntRef<'a> {
    inner: &'a mfem_sys::ArrayInt,
}

impl ArrayInt {
    pub fn new() -> Self {
        let inner = mfem_sys::arrayint_with_len(0);
        Self { inner }
    }

    pub fn with_len(len: usize) -> Self {
        let inner = mfem_sys::arrayint_with_len(len as i32);
        Self { inner }
    }

    pub fn set_all(&mut self, value: i32) {
        // TODO: mfem_sys::ArrayInt_SetAll(self.inner.pin_mut(), value);
        let slice: &mut [i32] = self.as_slice_mut();
        for entry in slice {
            *entry = value;
        }
    }
}

impl<'a> ArrayIntRef<'a> {
    pub fn as_slice(&self) -> &[i32] {
        let data = self.inner.GetData();
        let size = self.inner.Size() as usize;
        unsafe { std::slice::from_raw_parts(data, size) }
    }

    pub fn as_slice_mut(&self) -> &mut [i32] {
        let data = self.inner.GetDataMut();
        let size = self.inner.Size() as usize;
        unsafe { std::slice::from_raw_parts_mut(data, size) }
    }

    pub fn iter(&self) -> impl Iterator<Item = &i32> {
        self.as_slice().iter()
    }
}

////////////
// Vector //
////////////

pub struct Vector {
    inner: UniquePtr<mfem_sys::Vector>,
}

impl Vector {
    pub fn new() -> Self {
        let inner = UniquePtr::emplace(Vector::new());
        Self { inner }
    }
}

//////////
// Mesh //
//////////

pub struct Mesh {
    inner: UniquePtr<mfem_sys::Mesh>,
}

impl Mesh {
    pub fn new() -> Self {
        let inner = mfem_sys::Mesh_ctor();
        Self { inner }
    }

    pub fn from_file(path: &str) -> Result<Self, MfemError> {
        let generate_edges = 1;
        let refine = 1;
        let fix_orientation = true;
        let_cxx_string!(mesh_path = path);
        let inner =
            mfem_sys::Mesh_ctor_file(&mesh_path, generate_edges, refine, fix_orientation);
        Ok(Self { inner })
    }

    pub fn dimension(&self) -> i32 {
        self.inner.Dimension()
    }

    pub fn get_num_elems(&self) -> i32 {
        self.inner.GetNE()
    }

    pub fn get_nodes<'fes, 'a: 'fes>(&'a self) -> Option<GridFunctionRef<'fes, 'a>> {
        mfem_sys::Mesh_GetNodes(&self.inner)
            .ok()
            .map(|grid_func| GridFunctionRef { inner: grid_func })
    }

    pub fn get_bdr_attributes<'a>(&'a self) -> ArrayIntRef<'a> {
        let inner = mfem_sys::Mesh_bdr_attributes(&self.inner);
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

pub use mfem_sys::BasisType;

/////////////////////////////
// FiniteElementCollection //
/////////////////////////////

pub trait FiniteElementCollection: AsBase<mfem_sys::FiniteElementCollection> {
    fn get_name(&self) -> String {
        let ptr = self.as_base().Name();
        assert!(!ptr.is_null());
        let name = unsafe { std::ffi::CStr::from_ptr(ptr) };
        name.to_owned().into_string().expect("Valid string")
    }
}

impl FiniteElementCollection for mfem_sys::FiniteElementCollection {}

/////////////////////
// H1_FECollection //
/////////////////////

pub struct H1FeCollection {
    inner: UniquePtr<mfem_sys::H1_FECollection>,
}

impl H1FeCollection {
    pub fn new(p: i32, dim: i32, btype: BasisType) -> Self {
        let inner = mfem_sys::H1_FECollection_ctor(p, dim, btype.repr);
        Self { inner }
    }
}

impl FiniteElementCollection for H1FeCollection {}

impl AsBase<mfem_sys::FiniteElementCollection> for H1FeCollection {
    fn as_base(&self) -> &mfem_sys::FiniteElementCollection {
        mfem_sys::H1_FECollection_as_FEC(&self.inner)
    }
}

////////////////////////
// FiniteElementSpace //
////////////////////////

pub use mfem_sys::Ordering_Type as OrderingType;

pub struct FiniteElementSpace<'mesh, 'fec> {
    inner: UniquePtr<mfem_sys::FiniteElementSpace<'mesh, 'fec>>,
}

impl<'mesh, 'fec> FiniteElementSpace<'mesh, 'fec> {
    pub fn new(
        mesh: &'mesh Mesh,
        fec: &'fec dyn FiniteElementCollection,
        vdim: i32,
        ordering: OrderingType,
    ) -> Self {
        let inner =
            mfem_sys::FiniteElementSpace_ctor(&mesh.inner, &fec.as_base(), vdim, ordering);
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
        mfem_sys::FiniteElementSpace_GetEssentialTrueDofs(
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
    inner: UniquePtr<mfem_sys::GridFunction<'fes>>,
}

pub struct GridFunctionRef<'fes, 'a> {
    inner: &'a mfem_sys::GridFunction<'fes>,
}

impl<'fes> GridFunction<'fes> {
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        let inner = mfem_sys::GridFunction_ctor_fes(&fespace.inner);
        Self { inner }
    }

    /// Project `coeff` [`Coefficient`] to this [`GridFunction`].
    ///
    /// The projection computation depends on the choice of the [`FiniteElementSpace`] `fespace`.
    ///
    /// Note that this is usually interpolation at the degrees of freedom in each element (not L2 projection).
    pub fn project_coefficient(&mut self, coeff: &dyn Coefficient) {
        mfem_sys::GridFunction_ProjectCoefficient(self.inner.pin_mut(), coeff.as_base());
    }

    pub fn set_all(&mut self, value: f64) {
        mfem_sys::GridFunction_SetAll(self.inner.pin_mut(), value);
    }

    pub fn save_to_file(&self, path: &str, precision: i32) {
        let_cxx_string!(fname = path);
        mfem_sys::GridFunction_Save(&self.inner, &fname, precision);
    }
}

impl<'fes, 'a> GridFunctionRef<'fes, 'a> {
    pub fn get_own_fec(&self) -> Option<&dyn FiniteElementCollection> {
        mfem_sys::GridFunction_OwnFEC(self.inner)
            .ok()
            .map(|fec| fec as &dyn FiniteElementCollection)
    }
}

impl<'fes> VectorLike for GridFunction<'fes> {}

impl<'fes> AsBase<mfem_sys::Vector> for GridFunction<'fes> {
    fn as_base(&self) -> &mfem_sys::Vector {
        mfem_sys::GridFunction_as_Vector(&self.inner)
    }
}

impl<'fes> AsBaseMut<mfem_sys::Vector> for GridFunction<'fes> {
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::Vector> {
        mfem_sys::GridFunction_as_mut_Vector(self.inner.pin_mut())
    }
}

////////////////
// LinearForm //
////////////////

pub struct LinearForm<'fes> {
    inner: UniquePtr<mfem_sys::LinearForm<'fes>>,
}

impl<'fes> LinearForm<'fes> {
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        let inner = mfem_sys::LinearForm_ctor_fes(&fespace.inner);
        Self { inner }
    }

    pub fn add_domain_integrator<Lfi>(&mut self, lfi: Lfi)
    where
        Lfi: LinearFormIntegrator,
    {
        mfem_sys::LinearForm_AddDomainIntegrator(self.inner.pin_mut(), lfi.into_base());
    }

    pub fn assemble(&mut self) {
        self.inner.pin_mut().Assemble();
    }
}

impl<'fes> VectorLike for LinearForm<'fes> {}

impl<'fes> AsBase<mfem_sys::Vector> for LinearForm<'fes> {
    fn as_base(&self) -> &mfem_sys::Vector {
        mfem_sys::LinearForm_as_Vector(&self.inner)
    }
}

impl<'fes> AsBaseMut<mfem_sys::Vector> for LinearForm<'fes> {
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::Vector> {
        mfem_sys::LinearForm_as_mut_Vector(self.inner.pin_mut())
    }
}

/////////////////
// Coefficient //
/////////////////

pub trait Coefficient: AsBase<mfem_sys::Coefficient> {
    // TODO(mkovaxx)
}

/////////////////////////
// ConstantCoefficient //
/////////////////////////

pub struct ConstantCoefficient {
    inner: UniquePtr<mfem_sys::ConstantCoefficient>,
}

impl ConstantCoefficient {
    pub fn new(value: f64) -> Self {
        let inner = mfem_sys::ConstantCoefficient_ctor(value);
        Self { inner }
    }
}

impl Coefficient for ConstantCoefficient {}

impl AsBase<mfem_sys::Coefficient> for ConstantCoefficient {
    fn as_base(&self) -> &mfem_sys::Coefficient {
        mfem_sys::ConstantCoefficient_as_Coeff(&self.inner)
    }
}

//////////////////////////
// LinearFormIntegrator //
//////////////////////////

pub trait LinearFormIntegrator:
    AsBase<mfem_sys::LinearFormIntegrator>
    + IntoBase<UniquePtr<mfem_sys::LinearFormIntegrator>>
{
    // TODO(mkovaxx)
}

////////////////////////
// DomainLFIntegrator //
////////////////////////

pub struct DomainLFIntegrator<'coeff> {
    inner: UniquePtr<mfem_sys::DomainLFIntegrator<'coeff>>,
}

impl<'coeff> DomainLFIntegrator<'coeff> {
    pub fn new(coeff: &'coeff dyn Coefficient, a: i32, b: i32) -> Self {
        let inner = mfem_sys::DomainLFIntegrator_ctor_ab(coeff.as_base(), a, b);
        Self { inner }
    }
}

impl<'coeff> LinearFormIntegrator for DomainLFIntegrator<'coeff> {}

impl<'coeff> AsBase<mfem_sys::LinearFormIntegrator> for DomainLFIntegrator<'coeff> {
    fn as_base(&self) -> &mfem_sys::LinearFormIntegrator {
        mfem_sys::DomainLFIntegrator_as_LFI(&self.inner)
    }
}

impl<'coeff> IntoBase<UniquePtr<mfem_sys::LinearFormIntegrator>>
    for DomainLFIntegrator<'coeff>
{
    fn into_base(self) -> UniquePtr<mfem_sys::LinearFormIntegrator> {
        mfem_sys::DomainLFIntegrator_into_LFI(self.inner)
    }
}

//////////////////
// BilinearForm //
//////////////////

pub struct BilinearForm<'fes> {
    inner: UniquePtr<mfem_sys::BilinearForm<'fes>>,
}

impl<'fes> BilinearForm<'fes> {
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        let inner = mfem_sys::BilinearForm_ctor_fes(&fespace.inner);
        Self { inner }
    }

    pub fn add_domain_integrator<Bfi>(&mut self, bfi: Bfi)
    where
        Bfi: BilinearFormIntegrator,
    {
        mfem_sys::BilinearForm_AddDomainIntegrator(self.inner.pin_mut(), bfi.into_base());
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
        mfem_sys::BilinearForm_FormLinearSystem(
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
    AsBase<mfem_sys::BilinearFormIntegrator>
    + IntoBase<UniquePtr<mfem_sys::BilinearFormIntegrator>>
{
    // TODO(mkovaxx)
}

/////////////////////////
// DiffusionIntegrator //
/////////////////////////

pub struct DiffusionIntegrator<'coeff> {
    inner: UniquePtr<mfem_sys::DiffusionIntegrator<'coeff>>,
}

impl<'coeff> DiffusionIntegrator<'coeff> {
    pub fn new(coeff: &'coeff dyn Coefficient) -> Self {
        let inner = mfem_sys::DiffusionIntegrator_ctor(coeff.as_base());
        Self { inner }
    }
}

impl<'coeff> BilinearFormIntegrator for DiffusionIntegrator<'coeff> {}

impl<'coeff> AsBase<mfem_sys::BilinearFormIntegrator> for DiffusionIntegrator<'coeff> {
    fn as_base(&self) -> &mfem_sys::BilinearFormIntegrator {
        mfem_sys::DiffusionIntegrator_as_BFI(&self.inner)
    }
}

impl<'coeff> IntoBase<UniquePtr<mfem_sys::BilinearFormIntegrator>>
    for DiffusionIntegrator<'coeff>
{
    fn into_base(self) -> UniquePtr<mfem_sys::BilinearFormIntegrator> {
        mfem_sys::DiffusionIntegrator_into_BFI(self.inner)
    }
}

//////////////
// Operator //
//////////////

pub trait Operator: AsBase<mfem_sys::Operator> {
    fn height(&self) -> i32 {
        self.as_base().Height()
    }
}

////////////////////
// OperatorHandle //
////////////////////

pub use mfem_sys::Operator_Type as OperatorType;

pub struct OperatorHandle {
    inner: UniquePtr<mfem_sys::OperatorHandle>,
}

impl OperatorHandle {
    pub fn new() -> Self {
        let inner = mfem_sys::OperatorHandle_ctor();
        Self { inner }
    }

    pub fn get_type(&self) -> OperatorType {
        self.inner.Type()
    }
}

impl Operator for OperatorHandle {}

impl AsBase<mfem_sys::Operator> for OperatorHandle {
    fn as_base(&self) -> &mfem_sys::Operator {
        mfem_sys::OperatorHandle_as_ref(&self.inner)
    }
}

//////////////////
// SparseMatrix //
//////////////////

pub struct SparseMatrix {
    inner: UniquePtr<mfem_sys::SparseMatrix>,
}

impl<'a> TryFrom<OperatorHandle> for SparseMatrix {
    type Error = MfemError;

    fn try_from(value: OperatorHandle) -> Result<Self, Self::Error> {
        todo!()
    }
}

pub struct SparseMatrixRef<'a> {
    inner: &'a mfem_sys::SparseMatrix,
}

impl<'a> TryFrom<&'a OperatorHandle> for SparseMatrixRef<'a> {
    // TODO(mkovaxx)
    type Error = MfemError;

    fn try_from(value: &'a OperatorHandle) -> Result<Self, Self::Error> {
        let inner =
            mfem_sys::OperatorHandle_try_as_SparseMatrix(&value.inner).map_err(|_| {
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

pub trait Solver: AsBaseMut<mfem_sys::Solver> {
    // TODO(mkovaxx)
}

////////////////
// GSSmoother //
////////////////

pub struct GsSmoother<'mat> {
    inner: UniquePtr<mfem_sys::GSSmoother<'mat>>,
}

impl<'mat> GsSmoother<'mat> {
    pub fn new(a: &SparseMatrixRef<'mat>, t: i32, it: i32) -> Self {
        let inner = mfem_sys::GSSmoother_ctor(a.inner, t, it);
        Self { inner }
    }
}

impl<'mat> Solver for GsSmoother<'mat> {}

impl<'mat> AsBaseMut<mfem_sys::Solver> for GsSmoother<'mat> {
    fn as_base_mut(&mut self) -> std::pin::Pin<&mut mfem_sys::Solver> {
        mfem_sys::GSSmoother_as_mut_Solver(self.inner.pin_mut())
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
    mfem_sys::PCG(
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
