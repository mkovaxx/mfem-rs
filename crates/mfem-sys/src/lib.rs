#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        // https://github.com/dtolnay/cxx/issues/280

        include!("mfem-sys/include/wrapper.hpp");

        //////////
        // MESH //
        //////////

        type Mesh;

        #[cxx_name = "construct_unique"]
        pub fn Mesh_ctor() -> UniquePtr<Mesh>;

        #[cxx_name = "construct_unique"]
        pub fn Mesh_ctor_file(filename: &CxxString) -> UniquePtr<Mesh>;

        #[cxx_name = "Mesh_Dimension"]
        pub fn Mesh_Dimension(mesh: &Mesh) -> i32;

        #[cxx_name = "Mesh_GetNE"]
        pub fn Mesh_GetNE(mesh: &Mesh) -> i32;

        #[cxx_name = "Mesh_UniformRefinement"]
        pub fn Mesh_UniformRefinement(mesh: Pin<&mut Mesh>, ref_algo: i32);
    }
}
