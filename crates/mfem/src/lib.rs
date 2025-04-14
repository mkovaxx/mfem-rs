use autocxx::moveit::MakeCppStorage;
use autocxx::prelude::*;
use cxx::{let_cxx_string, memory::UniquePtrTarget};
use paste::paste;
use std::{
    borrow::Cow, fmt, marker::PhantomData, path::Path, pin::Pin, ptr, slice,
};

/// *Pointers* to C++ objects are wrapped by this macro.
macro_rules! wrap_mfem_sys {
    ($(#[$doc: meta])* $name: ident <$($l: lifetime)?>) => {
        wrap_mfem_sys!($(#[$doc])* $name <$($l)?> ($name));
    };

    ($(#[$doc: meta])* $name: ident <$($l: lifetime)?>
        ($sys_name: ident)
    ) => {
        $(#[$doc])*
        #[repr(transparent)]
        #[allow(non_camel_case_types)]
        // An owned value, wrapping the same C++ value.  We wrap the
        // pointer itself (not the C++ value) in order to avoid the
        // problems with Rust references to C++ values which may not
        // hold all the required Rust invariants for references (see
        // autocxx::CppRef for more information).
        pub struct $name $(<$l>)?{
            ptr: *mut mfem_sys::$sys_name,
            marker: PhantomData<$(&$l)? ()>,
        }

        impl $(<$l>)? Drop for $name $(<$l>)?
        where mfem_sys::$sys_name: UniquePtrTarget {
            fn  drop(&mut self) {
                drop(unsafe { UniquePtr::from_raw(self.ptr) });
            }
        }

        unsafe impl $(<$l>)? RefTarget for $name $(<$l>)? {
            type Target = mfem_sys::$sys_name;

            #[inline]
            fn __wrap_ptr(ptr: *mut Self::Target) -> Self {
                debug_assert!(!ptr.is_null());
                Self { ptr, marker: PhantomData }
            }

            #[inline]
            fn __unwrap_ptr(x: &Self) -> *mut Self::Target {
                x.ptr
            }
        }
     };
}

/// Trait bound for types `T` for which [`Ref`] or [`Mut`] can be
/// created.
pub unsafe trait RefTarget {
    /// The C++ value (given by autocxx) the target points to.
    #[doc(hidden)]
    type Target: UniquePtrTarget;

    /// Wrap the pointer to a C++ value.
    #[doc(hidden)]
    fn __wrap_ptr(ptr: *mut Self::Target) -> Self;

    /// Return the pointer to the C++ value.
    #[doc(hidden)]
    fn __unwrap_ptr(x: &Self) -> *mut Self::Target;
}

/// Private trait for creation and de-reference for all created wrappers.
trait Owned: RefTarget {
    /// Create an owned value from the pointer `ptr`.
    fn from_uniqueptr(ptr: UniquePtr<Self::Target>) -> Self;

    fn emplace<N>(n: N) -> Self
    where
        N: New<Output = Self::Target>,
        Self::Target: MakeCppStorage;

    /// Takes ownership of the value and convert it to a pointer.
    /// Suitable when the C++ function takes ownership of the pointer.
    fn into_raw(self) -> *mut Self::Target;

    /// Get a regular Rust reference out of this C++ reference.
    ///
    /// # Safety
    ///
    /// Callers must guarantee that the referent is not modified by
    /// any other C++ or Rust code while the returned reference
    /// exists. Callers must also guarantee that no mutable Rust
    /// reference is created to the referent while the returned
    /// reference exists.
    ///
    /// Callers must also be sure that the C++ reference is properly
    /// aligned, not null, pointing to valid data, etc.
    fn as_mfem(&self) -> &Self::Target;

    /// Get a regular Rust mutable reference out of this C++ reference.
    ///
    /// # Safety
    ///
    /// Callers must guarantee that the referent is not modified by
    /// any other C++ or Rust code while the returned reference
    /// exists.  Callers must also guarantee that no other Rust
    /// reference is created to the referent while the returned
    /// reference exists.
    fn as_mfem_mut(&mut self) -> Pin<&mut Self::Target>;

    /// Temporary workaround until it becomes clear how to
    /// handle shared mutability of some MFEM objects.
    unsafe fn as_mfem_internal_ptr(&self) -> *mut Self::Target;
}

impl<T: RefTarget> Owned for T {
    #[inline]
    fn from_uniqueptr(ptr: UniquePtr<T::Target>) -> T {
        T::__wrap_ptr(ptr.into_raw())
    }

    fn emplace<N>(n: N) -> T
    where
        N: New<Output = T::Target>,
        T::Target: MakeCppStorage,
    {
        Self::from_uniqueptr(UniquePtr::emplace(n))
    }

    fn into_raw(self) -> *mut T::Target {
        let ptr = T::__unwrap_ptr(&self);
        std::mem::forget(self);
        ptr
    }

    #[inline]
    fn as_mfem(&self) -> &T::Target {
        unsafe { &*T::__unwrap_ptr(self) }
    }

    #[inline]
    fn as_mfem_mut(&mut self) -> Pin<&mut T::Target> {
        unsafe { Pin::new_unchecked(&mut *T::__unwrap_ptr(self)) }
    }

    #[inline]
    unsafe fn as_mfem_internal_ptr(&self) -> *mut T::Target {
        T::__unwrap_ptr(self) as *mut _
    }
}

/// Immutable reference to `T`.
#[must_use]
pub struct Ref<'a, T: RefTarget> {
    // Some functions return a *const mfem_sys::T that we cannot
    // reinterpret as a reference to T (see `wrap_mfem_sys`) because
    // that would reference a value local to the function.  So we wrap
    // the pointer in `Self` that deref to &T.  `Self` does *not* own
    // the value pointed by the pointer in `T`, so dropping this
    // reference must *not* call `drop` on `ptr`.
    ptr: *const T::Target,
    marker: PhantomData<&'a T>,
}

impl<'a, T: RefTarget> Ref<'a, T> {
    fn from_ptr(ptr: *const T::Target) -> Self {
        Self {
            ptr,
            marker: PhantomData,
        }
    }

    fn from_ref(r: &T::Target) -> Self {
        Self::from_ptr(r as *const _)
    }

    fn as_ref(&self) -> &'a T {
        // Since `wrap_mfem_sys` wraps the pointer in a transparent
        // way, we can reinterpret it (the change in mutability of the
        // pointer is fine as we return an immutable reference).
        unsafe { std::mem::transmute::<&*const T::Target, &T>(&self.ptr) }
    }
}

impl<'a, T: RefTarget> std::ops::Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T: RefTarget + ToOwned> From<Ref<'a, T>> for Cow<'a, T> {
    fn from(value: Ref<'a, T>) -> Self {
        Self::Borrowed(value.as_ref())
    }
}

/// Mutable reference to `T`.
#[must_use]
pub struct Mut<'a, T: RefTarget> {
    ptr: *mut T::Target,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T: RefTarget> std::ops::Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute::<&*mut T::Target, &T>(&self.ptr) }
    }
}

impl<'a, T: RefTarget> std::ops::DerefMut for Mut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::mem::transmute::<&mut *mut T::Target, &mut T>(&mut self.ptr)
        }
    }
}

impl<'a, T: RefTarget> Mut<'a, T> {
    // fn from_ptr(ptr: *mut T::Target) -> Self {
    //     let ptr = T::__wrap_ptr(ptr);
    //     Self { ptr, marker: PhantomData }
    // }

    // fn from_ref(r: &mut T::Target) -> Self {
    //     Self::from_ptr(r as *mut _)
    // }
}

// Subclass relationships.  These cannot be declared as blanket
// implementations because the type must be local.
macro_rules! subclass {
    ($name: ident $(<$l: lifetime>)?, $parent: ident) => {
        subclass!($name $(<$l>)? ($name), $parent ($parent));
    };

    ($name: ident $(<$l: lifetime>)? ($sys_name:ident),
        $parent: ident ($sys_parent: ident)
    ) => {
        impl $(<$l>)? std::ops::Deref for $name $(<$l>)? {
            type Target = $parent $(<$l>)?;

            fn deref(&self) -> &Self::Target {
                // $name is a transparent wrapper of a pointer to
                // mfem_sys::$name.  We can just reinterpret it as `Parent`.
                unsafe { std::mem::transmute::<&$name, &$parent>(self) }
            }
        }

        impl $(<$l>)? std::ops::DerefMut for $name $(<$l>)? {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { std::mem::transmute::<&mut $name, &mut $parent>(self) }
            }
        }

        subclass_from!($name $(<$l>)? ($sys_name), $parent ($sys_parent));
    };
}

macro_rules! subclass_from {
    ($name: ident $(<$l: lifetime>)?, $parent: ident) => {
        subclass_from!($name $(<$l>)? ($name), $parent ($parent));
    };

    ($name: ident $(<$l: lifetime>)? ($sys_name:ident),
        $parent: ident ($sys_parent: ident)
    ) => {
        impl $(<$l>)? From<$name $(<$l>)?> for $parent $(<$l>)? {
            fn from(value: $name $(<$l>)?) -> Self {
                unsafe { std::mem::transmute::<$name, $parent>(value) }
            }
        }

        // Since conversions reinterpret pointers, declaring a wrong
        // subclass relationship results in UB.  Thus add a test
        // making sure that the C++ conversion function exists.
        paste! {
            #[cfg(test)]
            #[test]
            #[allow(non_snake_case)]
            fn [<test_ $name _as_ $parent>]() {
                fn _convert(
                    x: *const mfem_sys::$sys_name
                ) -> *const mfem_sys::$sys_parent {
                    unsafe { mfem_sys:: [<$sys_name _as_ $sys_parent>](x) }
                }
            }
        }
    };
}

// FIXME: Array<T> by using a trait ArrayTarget on T.
wrap_mfem_sys! {
    /// An Array of `i32`.
    ArrayInt<>
}

impl Clone for ArrayInt {
    fn clone(&self) -> Self {
        Self::from_uniqueptr(mfem_sys::arrayint_copy(self.as_mfem()))
    }
}

impl std::ops::Deref for ArrayInt {
    type Target = [i32];

    fn deref(&self) -> &Self::Target {
        let len = self.len();
        if len == 0 {
            &[]
        } else {
            let data = self.as_mfem().GetData();
            // autocxx::c_int is declared "transparent".
            let data = data as *const std::ffi::c_int;
            unsafe { slice::from_raw_parts(data, len) }
        }
    }
}

impl std::ops::DerefMut for ArrayInt {
    // Dereferencing to a mutable slice does not allow to move the
    // target because the slice is not `Sized` (so, for example,
    // `swap` cannot be used).
    #[inline]
    fn deref_mut(&mut self) -> &mut [i32] {
        let len = self.len();
        if len == 0 {
            &mut []
        } else {
            let data = self.as_mfem_mut().GetDataMut();
            let data = data as *mut std::ffi::c_int;
            unsafe { slice::from_raw_parts_mut(data, len) }
        }
    }
}

impl ArrayInt {
    #[must_use]
    pub fn new() -> Self {
        Self::from_uniqueptr(mfem_sys::arrayint_with_len(0))
    }

    #[must_use]
    pub fn with_len(len: usize) -> Self {
        let len = len.try_into().expect("Valid i32 len");
        Self::from_uniqueptr(mfem_sys::arrayint_with_len(len))
    }

    #[doc(alias = "Array::Size")]
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let len = self.as_mfem().Size();
        debug_assert!(len >= 0);
        len as usize
    }
}

////////////
// Vector //
////////////

wrap_mfem_sys! {
    /// Vector of [`f64`] numbers.
    ///
    /// A `Vector` may depend on data owned by another vector.  The
    /// lifetime reflects these possible dependencies.
    Vector<'dep>
}

impl<'a> std::ops::Deref for Vector<'a> {
    type Target = [f64];

    #[inline]
    fn deref(&self) -> &Self::Target {
        let len = Vector::len(self);
        if len == 0 {
            &[]
        } else {
            let data = self.as_mfem().GetData();
            unsafe { slice::from_raw_parts(data, len) }
        }
    }
}

impl<'a> std::ops::DerefMut for Vector<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = Vector::len(self);
        if len == 0 {
            &mut []
        } else {
            let data = self.as_mfem_mut().GetData();
            unsafe { slice::from_raw_parts_mut(data, len) }
        }
    }
}

impl Vector<'static> {
    #[must_use]
    pub fn new() -> Self {
        Self::emplace(mfem_sys::Vector::new())
    }
}

impl<'a> Vector<'a> {
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let l: i32 = self.as_mfem().Size().into();
        debug_assert!(l >= 0);
        l as usize
    }
}

//////////////
// Operator //
//////////////

wrap_mfem_sys! {
    Operator<'deps>
}

impl<'deps> Operator<'deps> {
    #[must_use]
    pub fn height(&self) -> usize {
        let h: i32 = self.as_mfem().Height();
        debug_assert!(h >= 0);
        h as usize
    }

    #[must_use]
    pub fn width(&self) -> usize {
        let w: i32 = self.as_mfem().Width();
        debug_assert!(w >= 0);
        w as usize
    }

    #[must_use]
    pub fn get_type(&self) -> OperatorType {
        self.as_mfem().GetType()
    }
}

////////////
// Matrix //
////////////

wrap_mfem_sys! {
    /// Represent a matrix.
    Matrix<'deps>
}

subclass!(Matrix<'deps>, Operator);

//////////
// Mesh //
//////////

wrap_mfem_sys! {
    Mesh<>
}

impl Mesh {
    #[must_use]
    pub fn new() -> Self {
        Self::emplace(mfem_sys::Mesh::new1())
    }

    /// Return a mesh created by reading a file in MFEM, Netgen, or
    /// VTK format.
    #[must_use]
    pub fn from_file(path: &str) -> Result<Self, MfemError> {
        let generate_edges = c_int(1);
        let refine = c_int(1);
        let fix_orientation = true;
        let_cxx_string!(mesh_path = path);
        Ok(Self::emplace(mfem_sys::Mesh::new6(
            &mesh_path,
            generate_edges,
            refine,
            fix_orientation,
        )))
    }

    #[must_use]
    pub fn dimension(&self) -> i32 {
        self.as_mfem().Dimension().into()
    }

    #[must_use]
    pub fn get_num_elems(&self) -> i32 {
        self.as_mfem().GetNE().into()
    }

    pub fn get_nodes<'a>(&'a self) -> Option<Ref<'a, GridFunction<'a>>> {
        let nodes = self.as_mfem().GetNodes2();
        if nodes.is_null() {
            None
        } else {
            Some(Ref::from_ptr(nodes))
        }
    }

    pub fn bdr_attributes(&self) -> Ref<'_, ArrayInt> {
        let attr = mfem_sys::Mesh_bdr_attributes(self.as_mfem());
        Ref::from_ref(attr)
    }

    pub fn uniform_refinement(&mut self, ref_algo: RefAlgo) {
        self.as_mfem_mut()
            .UniformRefinement1(c_int(ref_algo as i32))
    }

    pub fn save(&self) -> MeshSave<'_> {
        MeshSave {
            mesh: self,
            precision: 8,
        }
    }

    #[must_use]
    pub fn with_fec<'a, FEC>(&'a mut self, fec: FEC) -> MeshWithFEC<'a>
    where
        FEC: Fn(
            Option<Ref<'a, FiniteElementCollection>>,
        ) -> Cow<'a, FiniteElementCollection>,
    {
        let nodes = self.as_mfem().GetNodes2();
        // Safety: The mesh is stored alongside the FEC it may contain.
        let fec = if nodes.is_null() {
            fec(None)
        } else {
            let mesh_fec = Ref::<'a, GridFunction>::from_ptr(nodes).own_fec();
            fec(mesh_fec)
        };
        MeshWithFEC { mesh: self, fec }
    }
}

pub struct MeshWithFEC<'a> {
    mesh: &'a mut Mesh,
    // We use `Cow` for its ability to hold both an owned type and a
    // reference—not because it is clone on write.  One could have
    // defined our own type but this should already be familiar to the user.
    fec: Cow<'a, FiniteElementCollection>,
}

impl<'a> std::fmt::Debug for MeshWithFEC<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mesh = self.mesh.as_mfem() as *const _;
        let fec = self.fec.as_ref().as_mfem() as *const _;
        write!(f, "MeshWithFEC{{mesh: {:?}, fec: {:?}}}", mesh, fec)
    }
}

impl<'a> MeshWithFEC<'a> {
    #[must_use]
    pub fn mesh(&self) -> &Mesh {
        self.mesh
    }

    #[must_use]
    pub fn fec(&self) -> &FiniteElementCollection {
        &*self.fec
    }
}

#[must_use]
pub struct MeshSave<'a> {
    mesh: &'a Mesh,
    precision: i32,
}

impl<'a> MeshSave<'a> {
    pub fn precision(&self, p: i32) -> Self {
        Self {
            mesh: self.mesh,
            precision: p,
        }
    }

    #[inline]
    pub fn to_file(&self, path: impl AsRef<Path>) {
        let path = path.as_ref().as_os_str().as_encoded_bytes();
        let_cxx_string!(fname = path);
        self.mesh.as_mfem().Save(&fname, c_int(self.precision));
    }
}

/// Algorithm for [`Mesh::uniform_refinement`].
pub enum RefAlgo {
    /// Algorithm "A".
    ///
    /// Currently used only for pure tetrahedral meshes.
    /// Produces elements with better quality
    A = 0,
    /// Algorithm "B".
    B = 1,
}

pub use mfem_sys::BasisType;

/// Define how reference functions are mapped to physical space.
///
/// A reference function $û(x̂)$ can be mapped to a function $u(x)$ on
/// a general physical element in following ways:
///
/// - $x = T(x̂)$ is the image of the reference point $x̂$,
/// - $J = J(x̂)$ is the Jacobian matrix of the transformation T,
/// - $w = w(x̂) = \det(J)$ is the transformation weight factor for square J,
/// - $w = w(x̂) = \det(Jᵀ J)^{1/2}$ is the transformation weight factor
///   in general.
#[derive(Debug, Clone, Copy)]
pub enum MapType {
    /// Used to distinguish an unset MapType variable from the known
    /// values below.
    Unknown = -1,
    /// For scalar fields; preserves point values u(x) = û(x̂).
    Value = 0,
    /// For scalar fields; preserves volume integrals u(x) = (1/w) û(x̂).
    Integral = 1,
    /// For vector fields; preserves surface integrals of the normal
    /// component u(x) = (J/w) û(x̂).
    HDiv = 2,
    /// For vector fields; preserves line integrals of the tangential
    /// component $u(x) = J⁻ᵀ û(x̂)$ (square J),
    /// $u(x) = J (Jᵀ J)⁻¹ û(x̂)$ (general J).
    HCurl = 3,
}

#[cfg(test)]
#[test]
fn test_map_type() {
    use mfem_sys::FiniteElement_MapType::*;
    fn check(x: mfem_sys::FiniteElement_MapType) -> i32 {
        // Check exhaustiveness.
        match x {
            UNKNOWN_MAP_TYPE => UNKNOWN_MAP_TYPE as i32,
            VALUE => VALUE as i32,
            INTEGRAL => INTEGRAL as i32,
            H_DIV => H_DIV as i32,
            H_CURL => H_CURL as i32,
        }
    }
    assert_eq!(check(UNKNOWN_MAP_TYPE), MapType::Unknown as i32);
    assert_eq!(check(VALUE), MapType::Value as i32);
    assert_eq!(check(INTEGRAL), MapType::Integral as i32);
    assert_eq!(check(H_DIV), MapType::HDiv as i32);
    assert_eq!(check(H_CURL), MapType::HCurl as i32);
}

/////////////////////////////
// FiniteElementCollection //
/////////////////////////////

wrap_mfem_sys! {
    /// Collection of finite elements from the same family in multiple
    /// dimensions.  It matches the degrees of freedom of a
    /// [`FiniteElementSpace`] between elements, and provides the
    /// finite element restriction from an element to its boundary.
    FiniteElementCollection<>
}

impl Clone for FiniteElementCollection {
    fn clone(&self) -> Self {
        let p = self.get_order();
        let raw = self.as_mfem().Clone(c_int(p));
        Self::from_uniqueptr(unsafe { UniquePtr::from_raw(raw) })
    }
}

/// Continuity type: defines the continuity of the field across
/// element interfaces.
pub enum ContType {
    /// Field is continuous across element interfaces.
    Continuous,
    /// Tangential components of vector field.
    Tangential,
    /// Normal component of vector field.
    Normal,
    /// Field is discontinuous across element interfaces.
    Discontinuous,
}

impl ContType {
    fn from_int(c_int(c): c_int) -> Self {
        match c {
            0 => Self::Continuous,
            1 => Self::Tangential,
            2 => Self::Normal,
            3 => Self::Discontinuous,
            _ => panic!("mfem::ContType: invalid integer value {c}."),
        }
    }
}

impl FiniteElementCollection {
    #[must_use]
    pub fn get_name(&self) -> String {
        let cstr = self.as_mfem().Name();
        let cstr = unsafe { std::ffi::CStr::from_ptr(cstr) };
        cstr.to_owned().into_string().expect("Name must be ASCII")
    }

    #[must_use]
    pub fn get_cont(&self) -> ContType {
        ContType::from_int(self.as_mfem().GetContType())
    }

    #[must_use]
    pub fn get_order(&self) -> i32 {
        self.as_mfem().GetOrder().into()
    }

    /// Factory method: return a newly allocated
    /// [`FiniteElementCollection`] according to the given name.
    ///
    /// | FEC Name | Space | Order | BasisType | FiniteElement::MapT | Notes |
    /// | :------: | :---: | :---: | :-------: | :-----: | :---: |
    /// | H1_\[DIM\]_\[ORDER\] | H1 | * | 1 | VALUE | H1 nodal elements |
    /// | H1@\[BTYPE\]_\[DIM\]_\[ORDER\] | H1 | * | * | VALUE | H1 nodal elements |
    /// | H1Pos_\[DIM\]_\[ORDER\] | H1 | * | 1 | VALUE | H1 nodal elements |
    /// | H1Pos_Trace_\[DIM\]_\[ORDER\] | H^{1/2} | * | 2 | VALUE | H^{1/2}-conforming trace elements for H1 defined on the interface between mesh elements (faces,edges,vertices) |
    /// | H1_Trace_\[DIM\]_\[ORDER\] | H^{1/2} | * | 1 | VALUE | H^{1/2}-conforming trace elements for H1 defined on the interface between mesh elements (faces,edges,vertices) |
    /// | H1_Trace@\[BTYPE\]_\[DIM\]_\[ORDER\] | H^{1/2} | * | 1 | VALUE | H^{1/2}-conforming trace elements for H1 defined on the interface between mesh elements (faces,edges,vertices) |
    /// | ND_\[DIM\]_\[ORDER\] | H(curl) | * | 1 / 0 | H_CURL | Nedelec vector elements |
    /// | ND@\[CBTYPE\]\[OBTYPE\]_\[DIM\]_\[ORDER\] | H(curl) | * | * / * | H_CURL | Nedelec vector elements |
    /// | ND_Trace_\[DIM\]_\[ORDER\] | H^{1/2} | * | 1 / 0  | H_CURL | H^{1/2}-conforming trace elements for H(curl) defined on the interface between mesh elements (faces) |
    /// | ND_Trace@\[CBTYPE\]\[OBTYPE\]_\[DIM\]_\[ORDER\] | H^{1/2} | * | 1 / 0 | H_CURL | H^{1/2}-conforming trace elements for H(curl) defined on the interface between mesh elements (faces) |
    /// | ND_R1D_\[DIM\]_\[ORDER\] | H(curl) | * | 1 / 0 | H_CURL | 3D H(curl)-conforming Nedelec vector elements in 1D. |
    /// | ND_R2D_\[DIM\]_\[ORDER\] | H(curl) | * | 1 / 0 | H_CURL | 3D H(curl)-conforming Nedelec vector elements in 2D. |
    /// | RT_\[DIM\]_\[ORDER\] | H(div) | * | 1 / 0 | H_DIV | Raviart-Thomas vector elements |
    /// | RT@\[CBTYPE\]\[OBTYPE\]_\[DIM\]_\[ORDER\] | H(div) | * | * / * | H_DIV | Raviart-Thomas vector elements |
    /// | RT_Trace_\[DIM\]_\[ORDER\] | H^{1/2} | * | 1 / 0 | INTEGRAL | H^{1/2}-conforming trace elements for H(div) defined on the interface between mesh elements (faces) |
    /// | RT_ValTrace_\[DIM\]_\[ORDER\] | H^{1/2} | * | 1 / 0 | VALUE | H^{1/2}-conforming trace elements for H(div) defined on the interface between mesh elements (faces) |
    /// | RT_Trace@\[BTYPE\]_\[DIM\]_\[ORDER\] | H^{1/2} | * | 1 / 0 | INTEGRAL | H^{1/2}-conforming trace elements for H(div) defined on the interface between mesh elements (faces) |
    /// | RT_ValTrace@\[BTYPE\]_\[DIM\]_\[ORDER\] |  H^{1/2} | * | 1 / 0 | VALUE | H^{1/2}-conforming trace elements for H(div) defined on the interface between mesh elements (faces) |
    /// | RT_R1D_\[DIM\]_\[ORDER\] | H(div) | * | 1 / 0 | H_DIV | 3D H(div)-conforming Raviart-Thomas vector elements in 1D. |
    /// | RT_R2D_\[DIM\]_\[ORDER\] | H(div) | * | 1 / 0 | H_DIV | 3D H(div)-conforming Raviart-Thomas vector elements in 2D. |
    /// | L2_\[DIM\]_\[ORDER\] | L2 | * | 0 | VALUE | Discontinuous L2 elements |
    /// | L2_T\[BTYPE\]_\[DIM\]_\[ORDER\] | L2 | * | 0 | VALUE | Discontinuous L2 elements |
    /// | L2Int_\[DIM\]_\[ORDER\] | L2 | * | 0 | INTEGRAL | Discontinuous L2 elements |
    /// | L2Int_T\[BTYPE\]_\[DIM\]_\[ORDER\] | L2 | * | 0 | INTEGRAL | Discontinuous L2 elements |
    /// | DG_Iface_\[DIM\]_\[ORDER\] | - | * | 0 | VALUE | Discontinuous elements on the interface between mesh elements (faces) |
    /// | DG_Iface@\[BTYPE\]_\[DIM\]_\[ORDER\] | - | * | 0 | VALUE | Discontinuous elements on the interface between mesh elements (faces) |
    /// | DG_IntIface_\[DIM\]_\[ORDER\] | - | * | 0 | INTEGRAL | Discontinuous elements on the interface between mesh elements (faces) |
    /// | DG_IntIface@\[BTYPE\]_\[DIM\]_\[ORDER\] | - | * | 0 | INTEGRAL | Discontinuous elements on the interface between mesh elements (faces) |
    /// | NURBS\[ORDER\] | - | * | - | VALUE | Non-Uniform Rational B-Splines (NURBS) elements |
    /// | LinearNonConf3D | - | 1 | 1 | VALUE | Piecewise-linear nonconforming finite elements in 3D |
    /// | CrouzeixRaviart | - | - | - | - | Crouzeix-Raviart nonconforming elements in 2D |
    /// | Local_\[FENAME\] | - | - | - | - | Special collection that builds a local version out of the FENAME collection |
    /// |-|-|-|-|-|-|
    /// | Linear | H1 | 1 | 1 | VALUE | Left in for backward compatibility, consider using H1_ |
    /// | Quadratic | H1 | 2 | 1 | VALUE | Left in for backward compatibility, consider using H1_ |
    /// | QuadraticPos | H1 | 2 | 2 | VALUE | Left in for backward compatibility, consider using H1_ |
    /// | Cubic | H1 | 2 | 1 | VALUE | Left in for backward compatibility, consider using H1_ |
    /// | Const2D | L2 | 0 | 1 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | Const3D | L2 | 0 | 1 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | LinearDiscont2D | L2 | 1 | 1 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | GaussLinearDiscont2D | L2 | 1 | 0 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | P1OnQuad | H1 | 1 | 1 | VALUE | Linear P1 element with 3 nodes on a square |
    /// | QuadraticDiscont2D | L2 | 2 | 1 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | QuadraticPosDiscont2D | L2 | 2 | 2 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | GaussQuadraticDiscont2D | L2 | 2 | 0 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | CubicDiscont2D | L2 | 3 | 1 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | LinearDiscont3D | L2 | 1 | 1 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | QuadraticDiscont3D | L2 | 2 | 1 | VALUE | Left in for backward compatibility, consider using L2_ |
    /// | ND1_3D | H(Curl) | 1 | 1 / 0 | H_CURL | Left in for backward compatibility, consider using ND_ |
    /// | RT0_2D | H(Div) | 1 | 1 / 0 | H_DIV | Left in for backward compatibility, consider using RT_ |
    /// | RT1_2D | H(Div) | 2 | 1 / 0 | H_DIV | Left in for backward compatibility, consider using RT_ |
    /// | RT2_2D | H(Div) | 3 | 1 / 0 | H_DIV | Left in for backward compatibility, consider using RT_ |
    /// | RT0_3D | H(Div) | 1 | 1 / 0 | H_DIV | Left in for backward compatibility, consider using RT_ |
    /// | RT1_3D | H(Div) | 2 | 1 / 0 | H_DIV | Left in for backward compatibility, consider using RT_ |
    ///
    #[must_use]
    pub fn new(name: &str) -> Self {
        unsafe {
            let c_name = name.as_ptr() as *const i8;
            // FIXME: aborts if the name is incorrect.
            let ptr = mfem_sys::FiniteElementCollection::New(c_name);
            let ptr = UniquePtr::from_raw(ptr);
            Self::from_uniqueptr(ptr)
        }
    }
}

/////////////////////
// H1_FECollection //
/////////////////////

wrap_mfem_sys! {
    /// Arbitrary order H1-conforming (continuous) finite element.
    /// Implements [`FiniteElementCollection`].
    #[must_use]
    H1_FECollection<>
}

subclass!(H1_FECollection, FiniteElementCollection);

impl From<H1_FECollection> for Cow<'_, FiniteElementCollection> {
    fn from(value: H1_FECollection) -> Self {
        Self::Owned(value.into())
    }
}

impl H1_FECollection {
    /// Return a H1-conforming (continuous) finite elements with order
    /// `p`, dimension `dim` and the default basis type
    /// [`GaussLobatto`][BasisType::GaussLobatto].
    pub fn new(p: i32, dim: i32) -> Self {
        Self::with_basis(p, dim, BasisType::GaussLobatto)
    }

    /// Return a H1-conforming (continuous) finite elements with
    /// positive basis functions with order `p` and dimension `dim`.
    #[doc(alias = "H1Pos_FECollection")]
    pub fn pos(p: i32, dim: i32) -> Self {
        // https://docs.mfem.org/html/fe__coll_8hpp_source.html#l00305
        Self::with_basis(p, dim, BasisType::Positive)
    }

    /// Return a H1-conforming (continuous) serendipity finite elements
    /// with order `p` and dimension `dim`.  Current implementation
    /// works in 2D only; 3D version is in development.
    #[doc(alias = "H1Ser_FECollection")]
    pub fn ser(p: i32, dim: i32) -> Self {
        Self::with_basis(p, dim, BasisType::Serendipity)
    }

    /// Return a H1-conforming (continuous) finite elements with order
    /// `p`, dimension `dim` and basis type `btype`.
    pub fn with_basis(p: i32, dim: i32, btype: BasisType) -> Self {
        Self::emplace(mfem_sys::H1_FECollection::new(
            c_int(p),
            c_int(dim),
            c_int(btype as i32),
        ))
    }

    /// Return a "H^{1/2}-conforming" trace finite elements with order
    /// `p` and dimension `dim` defined on the interface between mesh
    /// elements (faces,edges,vertices); these are the trace FEs of
    /// the H1-conforming FEs.
    #[doc(alias = "H1_Trace_FECollection")]
    pub fn trace(p: i32, dim: i32, btype: BasisType) -> Self {
        Self::with_basis(p, dim - 1, btype)
    }
}

/////////////////////
// L2_FECollection //
/////////////////////

#[allow(non_camel_case_types)]
pub type DG_FECollection = L2_FECollection;

wrap_mfem_sys! {
    /// Arbitrary order "L2-conforming" discontinuous finite elements.
    #[must_use]
    L2_FECollection<>
}

subclass!(L2_FECollection, FiniteElementCollection);

impl From<L2_FECollection> for Cow<'_, FiniteElementCollection> {
    fn from(value: L2_FECollection) -> Self {
        Self::Owned(value.into())
    }
}

impl L2_FECollection {
    pub fn new(p: i32, dim: i32) -> Self {
        Self::with_basis(p, dim, BasisType::GaussLegendre, MapType::Value)
    }

    pub fn with_basis(
        p: i32,
        dim: i32,
        btype: BasisType,
        map_type: MapType,
    ) -> Self {
        Self::emplace(mfem_sys::L2_FECollection::new(
            c_int(p),
            c_int(dim),
            c_int(btype as i32),
            c_int(map_type as i32),
        ))
    }
}

////////////////////////
// FiniteElementSpace //
////////////////////////

/// Ordering of degrees of freedom.
#[derive(Debug, Clone, Copy)]
pub enum Ordering {
    /// This ordering arranges the DOFs by nodes first.  It is often
    /// used for continuous finite element spaces, such as those based
    /// on H¹ elements.  This ordering is beneficial for certain types
    /// of solvers and preconditioners that exploit the nodal
    /// structure of the problem.
    ByNodes,
    /// This ordering arranges the DOFs by vector dimension first.  It
    /// is typically used for vector-valued problems where the
    /// components of the vector field are stored consecutively.  This
    /// can be useful for problems in elasticity or fluid dynamics
    /// where the vector field represents physical quantities like
    /// displacement or velocity.
    ByVdim,
}

impl From<Ordering> for mfem_sys::Ordering_Type {
    fn from(value: Ordering) -> Self {
        match value {
            Ordering::ByNodes => mfem_sys::Ordering_Type::byNODES,
            Ordering::ByVdim => mfem_sys::Ordering_Type::byVDIM,
        }
    }
}

wrap_mfem_sys! {
    /// Responsible for providing FEM view of the mesh, mainly managing
    /// the set of degrees of freedom.
    ///
    /// The term “degree of freedom”, or “dof” for short, can mean
    /// different things in different contexts.  In MFEM we use “dof”
    /// to refer to four closely related types of data; element dofs
    /// “edofs”, local dofs “ldofs”, true dofs “tdofs”, and vector
    /// dofs “vdofs”.  They are detailed below.
    ///
    /// ## Element DoF
    ///
    /// Element dofs, sometimes referred to as *edofs*, are the
    /// expansion coefficients used to build the linear combination of
    /// basis functions which approximate a field within one element
    /// of the computational mesh.  The arrangement of the element
    /// dofs is determined by the basis function and element
    /// types.
    ///
    /// Element dofs are usually accessed one element at a time but
    /// they can be concatenated together into a global vector when
    /// minimizing access time is crucial.  The global number of
    /// element dofs is not directly available from the
    /// [`FiniteElementSpace`].  It can be determined by repeatedly
    /// calling [`FiniteElementSpace::get_element_dofs`] and summing
    /// the lengths of the resulting `dofs` arrays.
    ///
    /// ## Local DoF
    ///
    /// Most basis function types share many of their element dofs
    /// with neighboring elements. Consequently, the global “edof”
    /// vector suggested above would contain many redundant entries.
    /// One of the primary roles of the FiniteElementSpace is to
    /// collapse out these redundancies and define a unique ordering
    /// of the remaining degrees of freedom.  The collapsed set of
    /// dofs are called *“local dofs”* or *ldofs* in the MFEM
    /// parlance.
    ///
    /// The term *local* in this context refers to the local rank in a
    /// parallel processing environment.  MFEM can, of course, be used
    /// in sequential computing environments but it is designed with
    /// parallel processing in mind and this terminology reflects that
    /// design focus.
    ///
    /// When running in parallel the set of local dofs contains all of
    /// the degrees of freedom associated with locally owned elements.
    /// When running in serial all elements are locally owned so all
    /// element dofs are represented in the set of local dofs.
    ///
    /// There are two important caveats regarding local dofs.  First,
    /// some basis function types, Nedelec and Raviart-Thomas are the
    /// prime examples, have an orientation associated with each basis
    /// function.  The relative orientations of such basis functions
    /// in neighboring elements can lead to shared degrees of freedom
    /// with opposite signs from the point of view of these
    /// neighboring elements.  MFEM typically chooses the orientation
    /// of the first such shared degree of freedom that it encounters
    /// as the default orientation for the corresponding local dof.
    /// When this local dof is referenced by a neighboring element
    /// which happens to require the opposite orientation the local
    /// dof index will be returned (by calls to functions such as
    /// FiniteElementSpace::GetElementDofs) as a negative integer. In
    /// such cases the actual offset into the vector of local dofs is
    /// *-index-1* and the value expected by this element should have
    /// the opposite sign to the value stored in the local dof
    /// vector.
    ///
    /// The second important caveat only pertains to high order
    /// Nedelec basis functions when shared triangular faces are
    /// present in the mesh.  In this very particular case the
    /// relative orientation of the face with respect to its two
    /// neighboring elements can lead to different definitions of the
    /// degrees of freedom associated with the interior of the face
    /// which cannot be handled by simply flipping the signs of the
    /// corresponding values.  The DofTransformation class is designed
    /// to manage the necessary @b edof to @b ldof transformations in
    /// this case.  In the majority of cases the DofTransformation is
    /// unnecessary and a NULL pointer will be returned in place of a
    /// pointer to this object. See DofTransformation for more
    /// information.
    ///
    /// ## True DoF
    ///
    /// As the name suggests “true dofs” or *tdofs* form the minimal
    /// set of data values needed (along with mesh and basis function
    /// definitions) to uniquely define a finite element
    /// discretization of a field.  The number of true dofs determines
    /// the size of the linear systems which typically need to be
    /// solved in FEM simulations.
    ///
    /// Often the true dofs and the local dofs are identical, however,
    /// there are important cases where they differ significantly.
    /// The first such case is related to non-conforming meshes.  On
    /// non-conforming meshes it is common for degrees of freedom
    /// associated with “hanging” nodes, edges, or faces to be
    /// constrained by degrees of freedom associated with another mesh
    /// entity.  In such cases the “hanging” degrees of freedom should
    /// not be considered “true” degrees of freedom since their values
    /// cannot be independently assigned.  For this reason the
    /// [`FiniteElementSpace`] must process these constraints and
    /// define a reduced set of “true” degrees of freedom which are
    /// distinct from the local degrees of freedom.
    ///
    /// The second important distinction arises in parallel
    /// processing.  When distributing a linear system in parallel
    /// each degree of freedom must be assigned to a particular
    /// processor, its owner.  From the finite element point of view
    /// it is convenient to distribute a computational mesh and define
    /// an owning processor for each element.  Since degrees of
    /// freedom may be shared between neighboring elements they may
    /// also be shared between neighboring processors.  Another role
    /// of the [`FiniteElementSpace`] is to identify the ownership of
    /// degrees of freedom which must be shared between processors.
    /// Therefore the set of “true” degrees of freedom must also
    /// remove redundant degrees of freedom which are owned by other
    /// processors.
    ///
    /// To summarize the set of true degrees of freedom are those
    /// degrees of freedom needed to solve a linear system
    /// representing the partial differential equation being modeled.
    /// True dofs differ from “local” dofs by eliminating redundancies
    /// across processor boundaries and applying the constraints
    /// needed to properly define fields on non-conforming meshes.
    ///
    /// ## Vector DoF
    ///
    /// Vector dofs or *vdofs* are related to fields which are
    /// constructed using multiple copies of the same set of basis
    /// functions.  A typical example would be the use of three
    /// instances of the scalar H1 basis functions to approximate the
    /// x, y, and z components of a displacement vector field in three
    /// dimensional space as often seen in elasticity
    /// simulations.
    ///
    /// Vector dofs do not represent a specific index space the way
    /// the three previous types of dofs do. Rather they are related
    /// to modifications of these other index spaces to accommodate
    /// multiple copies of the underlying function spaces.
    ///
    /// When using *vdofs*, i.e. when *vdim != 1*, the
    /// [`FiniteElementSpace`] only manages a single set of degrees of
    /// freedom and then uses simple rules to determine the
    /// appropriate offsets into the full index spaces.  Two ordering
    /// rules are supported; [`Ordering::ByNodes`] and
    /// [`Ordering::ByVdim`].
    ///
    /// Clearly the notion of a *vdof* is relevant in each of the
    /// three contexts mentioned above so extra care must be taken
    /// whenever *vdim ≠ 1* to ensure that the *edof*, *ldof*, or
    /// *tdof* is being interpreted correctly.
    FiniteElementSpace<'deps>
}

impl<'a> FiniteElementSpace<'a> {
    /// Return a new space from the `mesh` an Finite Element
    /// Collection `fec`.
    #[must_use]
    pub fn new<'mesh: 'a>(
        mesh_fec: &'a mut MeshWithFEC<'a>,
    ) -> FiniteElementSpaceBuilder<'a> {
        // The `fec` FiniteElementSpace will hold to `mesh` and `fec`.
        // Warning: one must be careful if one updates the `mesh`.
        FiniteElementSpaceBuilder {
            mesh: &mut mesh_fec.mesh,
            fec: mesh_fec.fec.as_ref(),
            vdim: 1,
            ordering: Ordering::ByNodes,
        }
    }

    /// Return the mesh.
    #[doc(alias = "GetMesh")]
    pub fn mesh(&self) -> Ref<'a, Mesh> {
        Ref::from_ptr(self.as_mfem().GetMesh())
    }

    pub fn fe_coll(&self) -> Ref<'a, FiniteElementCollection> {
        Ref::from_ptr(self.as_mfem().FEColl())
    }

    #[doc(alias = "Conforming")]
    #[must_use]
    pub fn is_conforming(&self) -> bool {
        self.as_mfem().Conforming()
    }

    #[doc(alias = "Nonconforming")]
    #[must_use]
    pub fn is_non_conforming(&self) -> bool {
        self.as_mfem().Nonconforming()
    }

    /// Return the number of vector true (conforming) dofs.
    #[must_use]
    pub fn get_true_vsize(&self) -> i32 {
        self.as_mfem().GetTrueVSize()
    }

    /// Returns vector dimension.
    #[must_use]
    pub fn get_vdim(&self) -> usize {
        let d = self.as_mfem().GetVDim();
        debug_assert!(d >= 0);
        d as usize
    }

    /// Returns indices of degrees of freedom of element `elem`.  The
    /// returned indices are offsets into an ldof vector.  See also
    /// [`Self::get_element_vdofs`].
    #[doc(alias = "GetElementDofs")]
    pub fn get_element_dofs(&self) {
        todo!()
    }

    /// Returns indices of degrees of freedom for the `i`'th element.
    /// The returned indices are offsets into an
    /// [ldof][FiniteElementSpace] vector with `vdim` not necessarily
    /// equal to 1.  The returned indices are always ordered
    /// [`ByNodes`][Ordering::ByNodes], irrespective of whether the
    /// space is [`ByNodes`][Ordering::ByNodes] or
    /// [`ByVdim`][Ordering::ByVdim].  See also [`Self::get_element_dofs`].
    pub fn get_element_vdofs(&self, i: i32, vdofs: &mut ArrayInt) {
        self.as_mfem().GetElementVDofs(i, vdofs.as_mfem_mut());
    }

    pub fn get_essential_true_dofs(
        &self,
        bdr_attr_is_ess: &ArrayInt,
        ess_tdof_list: &mut ArrayInt,
        component: Option<usize>,
    ) {
        self.as_mfem().GetEssentialTrueDofs(
            &bdr_attr_is_ess.as_mfem(),
            ess_tdof_list.as_mfem_mut(),
            component.map(|c| c as i32).unwrap_or(-1),
        );
    }
}

pub struct FiniteElementSpaceBuilder<'a> {
    mesh: &'a mut Mesh,
    fec: &'a FiniteElementCollection,
    vdim: i32,
    ordering: Ordering,
    //ext: NURBSExtension,
}

impl<'a> FiniteElementSpaceBuilder<'a> {
    #[must_use]
    pub fn build(&mut self) -> FiniteElementSpace<'a> {
        let ptr = mfem_sys::FES_new(
            self.mesh.as_mfem_mut(),
            self.fec.as_mfem(),
            c_int(self.vdim),
            self.ordering.into(),
        );
        Owned::from_uniqueptr(ptr)
    }

    /// The vector of the space have `vdim` components.  Default: `1`.
    pub fn vdim(&mut self, vdim: i32) -> &mut Self {
        self.vdim = vdim;
        self
    }

    /// Arranges the degrees of freedom of the FiniteElementSpace by
    /// `ordering`.  Default: [`Ordering::ByNodes`].
    pub fn ordering(&mut self, ordering: Ordering) -> &mut Self {
        self.ordering = ordering;
        self
    }
}

//////////////////
// GridFunction //
//////////////////

wrap_mfem_sys! {
    /// Represent a [`Vector`] with associated FE space.
    GridFunction<'fes>
}

subclass!(GridFunction<'fes>, Vector);

impl<'fes> GridFunction<'fes> {
    // XXX Can `fes` be mutated?
    // Hopefully so because it is shared with e.g. `Linearform`
    // XXX can you pass a different `fes` than the one for `Linearform`?
    /// Construct a GridFunction associated with the
    /// FiniteElementSpace `fes`.
    #[must_use]
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        Self::emplace(unsafe {
            mfem_sys::GridFunction::new2(fespace.as_mfem_internal_ptr())
        })
    }

    pub fn own_fec(&self) -> Option<Ref<'fes, FiniteElementCollection>> {
        // This pointer refers to some internals of `self`, whence
        // the lifetime in the above signature.
        // Modifying the C++ code, one confirms that this can be "const".
        let fec = mfem_sys::GridFunction_OwnFEC(self.as_mfem());
        if fec.is_null() {
            None
        } else {
            Some(Ref::from_ptr(fec))
        }
    }

    /// Project `coeff` [`Coefficient`] to this [`GridFunction`].
    ///
    /// The projection computation depends on the choice of the
    /// [`FiniteElementSpace`] `fespace`.
    ///
    /// Note that this is usually interpolation at the degrees of
    /// freedom in each element (not L2 projection).
    pub fn project_coefficient(&mut self, coeff: &mut VectorCoefficient) {
        self.as_mfem_mut().ProjectCoefficient1(coeff.as_mfem_mut());
    }

    pub fn save(&self) -> GridFunctionSave<'_> {
        GridFunctionSave {
            gf: self,
            precision: 8,
        }
    }
}

#[must_use]
pub struct GridFunctionSave<'a> {
    gf: &'a GridFunction<'a>,
    precision: i32,
}

impl GridFunctionSave<'_> {
    #[must_use]
    pub fn precision(&self, p: i32) -> Self {
        Self {
            gf: self.gf,
            precision: p,
        }
    }

    #[inline]
    pub fn to_file(&self, path: impl AsRef<Path>) {
        let path = path.as_ref().as_os_str().as_encoded_bytes();
        let_cxx_string!(filename = path);
        unsafe {
            let fname = filename.get_unchecked_mut().as_ptr();
            self.gf
                .as_mfem()
                .Save1(fname as *const i8, c_int(self.precision));
        }
    }
}

////////////////
// LinearForm //
////////////////

wrap_mfem_sys! {
    /// Vector with associated FE space and [`LinearFormIntegrator`]s.
    LinearForm<'fes>
}

subclass!(LinearForm<'deps>, Vector);

impl<'fes> LinearForm<'fes> {
    #[must_use]
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        let lfi = unsafe {
            mfem_sys::LinearForm::new1(fespace.as_mfem_internal_ptr())
        };
        Self::emplace(lfi)
    }

    pub fn fe_space(&self) -> Ref<'fes, FiniteElementSpace> {
        let raw = self.as_mfem().FESpace1();
        Ref::from_ptr(raw)
    }

    pub fn assemble(&mut self) {
        self.as_mfem_mut().Assemble();
    }

    pub fn add_domain_integrator<'deps: 'fes, Lfi>(&mut self, lfi: Lfi)
    where
        Lfi: Into<LinearFormIntegrator<'deps>>,
    {
        let lfi = lfi.into();
        unsafe {
            // The linear form "takes ownership of `lfi`".
            self.as_mfem_mut().AddDomainIntegrator(lfi.into_raw());
        }
    }
}

/////////////////
// Coefficient //
/////////////////

wrap_mfem_sys! {
    Coefficient<>
}

wrap_mfem_sys! {
    VectorCoefficient<>
}

/////////////////////////
// ConstantCoefficient //
/////////////////////////

wrap_mfem_sys! {
    ConstantCoefficient<>
}

subclass!(ConstantCoefficient, Coefficient);

impl ConstantCoefficient {
    #[must_use]
    pub fn new(c: f64) -> Self {
        Self::emplace(mfem_sys::ConstantCoefficient::new(c))
    }
}

//////////////////////////
// LinearFormIntegrator //
//////////////////////////

// `LinearFormIntegrator` is an abstract class on the C++ side.
// However, it will be taken by value by some functions such as
// `LinearForm::add_domain_integrator`.  Since
// `mfem::LinearFormIntegrator` lives on the C++ side and so can only
// be accessed through pointers, we define an owned type
// `LinearFormIntegrator`.  There will be no creation (`new`) function
// for that type but values can be produced by `Into` from owned
// values in sub-classes.
wrap_mfem_sys! {
    /// Common capabilities of `LinarFormIntegrator`s.
    LinearFormIntegrator<'deps>
}

////////////////////////
// DomainLFIntegrator //
////////////////////////

wrap_mfem_sys! {
    /// Represent a general integrator that supports delta coefficients.
    DeltaLFIntegrator<'deps>
}

subclass!(DeltaLFIntegrator<'deps>, LinearFormIntegrator);

////////////////////////
// DomainLFIntegrator //
////////////////////////

wrap_mfem_sys! {
    /// Type for domain integration L(v) := ∫ fv.
    DomainLFIntegrator<'coeff>
}

subclass!(DomainLFIntegrator<'coeff>, DeltaLFIntegrator);

// `Into` is not transitive, so implement this as well.
subclass_from!(DomainLFIntegrator<'coeff>, LinearFormIntegrator);

impl<'coeff> DomainLFIntegrator<'coeff> {
    /// Return a new linear form integrator v ↦ ∫ fv with order 2.
    #[must_use]
    pub fn new(qf: &'coeff mut Coefficient) -> Self {
        Self::with_order(qf, 2)
    }

    /// Return a new linear form integrator v ↦  ∫ fv with order `a`.
    #[must_use]
    pub fn with_order(qf: &'coeff mut Coefficient, a: usize) -> Self {
        // Safety: The result does not seem to take ownership of `qf`.
        let qf = qf.as_mfem_mut();
        let a = c_int(a as i32);
        let options = c_int(0);
        let lfi = mfem_sys::DomainLFIntegrator::new(qf, a, options);
        Self::emplace(lfi)
    }
}

//////////////////
// BilinearForm //
//////////////////

wrap_mfem_sys! {
    BilinearForm<'fes>
}
subclass!(BilinearForm<'fes>, Matrix);

impl<'fes> BilinearForm<'fes> {
    /// Creates bilinear form associated with Finite Element space `fespace`.
    #[must_use]
    pub fn new(fespace: &'fes FiniteElementSpace) -> Self {
        let b = unsafe {
            mfem_sys::BilinearForm::new2(fespace.as_mfem_internal_ptr())
        };
        Self::emplace(b)
    }

    /// Add new Domain Integrator.
    pub fn add_domain_integrator<'deps, Bfi>(&mut self, bfi: Bfi)
    where
        Bfi: Into<BilinearFormIntegrator<'deps>>,
    {
        let bfi = bfi.into();
        unsafe {
            // Doc says: "Assumes ownership of `bfi`".
            self.as_mfem_mut().AddDomainIntegrator(bfi.into_raw());
        }
    }

    pub fn assemble(&mut self, skip_zeros: bool) {
        self.as_mfem_mut()
            .Assemble(c_int(if skip_zeros { 1 } else { 0 }))
    }

    /// Form the linear system A X = B, corresponding to this bilinear
    /// form and the linear form `b`.  This method applies any
    /// necessary transformations to the linear system such as:
    /// eliminating boundary conditions; applying conforming
    /// constraints for non-conforming AMR; parallel assembly; static
    /// condensation; hybridization.
    ///
    /// The GridFunction-size vector `x` must contain the essential
    /// B.C.  The BilinearForm and the LinearForm-size vector `b` must
    /// be assembled.
    pub fn form_linear_system<'deps>(
        &mut self,
        ess_tdof_list: &ArrayInt,
        x: &mut Vector,
        b: &mut Vector,
        // FIXME: does the linear system depends on 'deps?
        a_mat: &mut OperatorHandle<'deps>,
        x_vec: &mut Vector,
        b_vec: &mut Vector,
        copy_interior: bool,
    ) {
        self.as_mfem_mut().FormLinearSystem(
            ess_tdof_list.as_mfem(),
            x.as_mfem_mut(),
            b.as_mfem_mut(),
            a_mat.as_mfem_mut(),
            x_vec.as_mfem_mut(),
            b_vec.as_mfem_mut(),
            c_int(if copy_interior { 1 } else { 0 }),
        );
    }

    pub fn recover_fem_solution(
        &mut self,
        x_vec: &Vector,
        b_vec: &Vector,
        x: &mut Vector,
    ) {
        self.as_mfem_mut().RecoverFEMSolution(
            x_vec.as_mfem(),
            b_vec.as_mfem(),
            x.as_mfem_mut(),
        );
    }
}

////////////////////////////
// BilinearFormIntegrator //
////////////////////////////

wrap_mfem_sys! {
    /// Common capabilities of `BilinearFormIntegrator`.
    BilinearFormIntegrator<'deps>
}

/////////////////////////
// DiffusionIntegrator //
/////////////////////////

wrap_mfem_sys! {
    /// α(Q ∇u, ∇v)
    DiffusionIntegrator<'coeff>
}
subclass!(DiffusionIntegrator<'coeff>, BilinearFormIntegrator);

impl<'coeff> DiffusionIntegrator<'coeff> {
    #[must_use]
    pub fn new() -> Self {
        let bfi = unsafe { mfem_sys::DiffusionIntegrator::new(ptr::null()) };
        Self::emplace(bfi)
    }

    #[must_use]
    pub fn with_coeff(coeff: &'coeff mut Coefficient) -> Self {
        let coeff = coeff.as_mfem_mut();
        let ir: *const mfem_sys::IntegrationRule = ptr::null();
        let bfi = unsafe { mfem_sys::DiffusionIntegrator::new1(coeff, ir) };
        Self::emplace(bfi)
    }
}

////////////////////
// OperatorHandle //
////////////////////

pub use mfem_sys::Operator_Type as OperatorType;

wrap_mfem_sys! {
    /// Pointer to an Operator of a specified type.
    ///
    /// This provides a common type for global, matrix-type operators
    /// to be used in bilinear forms, gradients of nonlinear forms,
    /// static condensation, hybridization, etc.
    OperatorHandle<'deps>
}

impl OperatorHandle<'static> {
    pub fn new() -> Self {
        Self::emplace(mfem_sys::OperatorHandle::new())
    }
}

// `OperatorHandle` is NOT a subclass of `Operator` but contains a
// pointer to an operator.  However, in C++, the operator *
// de-reference to `Operator` and `->` accesses `Operator` methods.
// In particular, where an `Operator&` is requested, a `*A`, where
// `A` is a `OperatorHandle`, can be provided.
//
// We express that dependence through `From`/`Into`.
impl<'a, 'deps> From<&'a OperatorHandle<'deps>> for Ref<'a, Operator<'deps>> {
    fn from(value: &'a OperatorHandle<'deps>) -> Self {
        value.op()
    }
}

impl<'deps> OperatorHandle<'deps> {
    #[inline]
    #[must_use]
    pub fn op(&self) -> Ref<'_, Operator<'deps>> {
        let o = mfem_sys::OperatorHandle_oper(self.as_mfem());
        Ref::from_ref(o)
    }

    #[must_use]
    pub fn get_type(&self) -> OperatorType {
        self.as_mfem().Type()
    }
}

//////////////////
// SparseMatrix //
//////////////////

wrap_mfem_sys! {
    /// Abstract data type for sparse matrices.
    AbstractSparseMatrix<>
}

wrap_mfem_sys! {
    /// Data type sparse matrix.
    SparseMatrix<>
}

subclass!(SparseMatrix, AbstractSparseMatrix);

wrap_mfem_sys! {
    BlockMatrix<>
}

subclass!(BlockMatrix, AbstractSparseMatrix);

impl<'a, 'deps: 'a> TryFrom<&'a OperatorHandle<'deps>>
    for Ref<'a, SparseMatrix>
{
    type Error = MfemError;

    fn try_from(o: &'a OperatorHandle<'deps>) -> Result<Self, Self::Error> {
        let ty = o.get_type();
        if ty == OperatorType::MFEM_SPARSEMAT {
            unsafe {
                let m = mfem_sys::OperatorHandle_as_SparseMatrix(o.as_mfem());
                Ok(Ref::from_ref(m))
            }
        } else {
            Err(MfemError::OperatorHandleTypeMismatch(
                ty,
                OperatorType::MFEM_SPARSEMAT,
            ))
        }
    }
}

////////////
// Solver //
////////////

wrap_mfem_sys! {
    /// Base structure for solvers.
    Solver<'deps>
}

wrap_mfem_sys! {
    /// Abstract data type for matrix inverse.
    MatrixInverse<'mat>
}
subclass!(MatrixInverse<'mat>, Solver);

wrap_mfem_sys! {
    SparseSmoother<'mat>
}

subclass!(SparseSmoother<'mat>, MatrixInverse);

////////////////
// GSSmoother //
////////////////

wrap_mfem_sys! {
    /// Data type for Gauss-Seidel smoother of sparse matrix.
    GSSmoother<'mat>
}

subclass!(GSSmoother<'mat>, SparseSmoother);

impl<'mat> GSSmoother<'mat> {
    #[must_use]
    pub fn new(t: i32, it: i32) -> Self {
        let gs = mfem_sys::GSSmoother::new(c_int(t), c_int(it));
        Self::emplace(gs)
    }

    #[must_use]
    pub fn with_matrix(a: &'mat SparseMatrix, t: i32, it: i32) -> Self {
        let gs = mfem_sys::GSSmoother::new1(a.as_mfem(), c_int(t), c_int(it));
        Self::emplace(gs)
    }

    /// Matrix vector multiplication with GS Smoother.
    pub fn mul(&self, x: &Vector, y: &mut Vector) {
        self.as_mfem().Mult(x.as_mfem(), y.as_mfem_mut());
    }
}

/////////
// PCG //
/////////

/// Preconditioned conjugate gradient method.
///
/// Remark: tolerances are squared.
pub fn pcg(
    a_mat: &Operator,
    solver: &mut Solver,
    b_vec: &Vector,
    x_vec: &mut Vector,
    print_iter: i32,
    max_num_iter: i32,
    rtol: f64,
    atol: f64,
) {
    mfem_sys::PCG(
        a_mat.as_mfem(),
        solver.as_mfem_mut(),
        &b_vec.as_mfem(),
        x_vec.as_mfem_mut(),
        print_iter,
        max_num_iter,
        mfem_sys::Real(rtol),
        mfem_sys::Real(atol),
    );
}

///////////
// Error //
///////////

#[derive(Debug)]
pub enum MfemError {
    // #[error("OperatorHandle type mismatch: expected {0:?} got {1:?}")]
    OperatorHandleTypeMismatch(OperatorType, OperatorType),
}

impl fmt::Display for MfemError {
    // FIXME: revise the output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperatorHandleTypeMismatch(t1, t2) => {
                write!(f, "OperatorHandleTypeMismatch({t1:?}, {t2:?})")
            }
        }
    }
}

impl std::error::Error for MfemError {}
