use cxx::{let_cxx_string, UniquePtr};
use thiserror::Error;

trait AsBase<T> {
    fn as_base(&self) -> &T;
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

    pub fn uniform_refinement(&mut self, ref_algo: RefAlgo) {
        self.inner.pin_mut().UniformRefinement(ref_algo as i32);
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
    fn get_name(&self) -> String;
}

impl FiniteElementCollection for mfem_sys::ffi::FiniteElementCollection {
    fn get_name(&self) -> String {
        let ptr = self.Name();
        assert!(!ptr.is_null());
        let name = unsafe { std::ffi::CStr::from_ptr(ptr) };
        name.to_owned().into_string().expect("Valid string")
    }
}

impl AsBase<mfem_sys::ffi::FiniteElementCollection> for mfem_sys::ffi::FiniteElementCollection {
    fn as_base(&self) -> &mfem_sys::ffi::FiniteElementCollection {
        self
    }
}

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

impl FiniteElementCollection for H1FeCollection {
    fn get_name(&self) -> String {
        self.as_base().get_name()
    }
}

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

impl<'fes> GridFunction<'fes> {}

impl<'fes, 'a> GridFunctionRef<'fes, 'a> {
    pub fn get_own_fec(&self) -> Option<&dyn FiniteElementCollection> {
        mfem_sys::ffi::GridFunction_OwnFEC(self.inner)
            .ok()
            .map(|fec| fec as &dyn FiniteElementCollection)
    }
}

#[derive(Error, Debug)]
pub enum MfemError {}
