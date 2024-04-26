#include "rust/cxx.h"

#include "mfem.hpp"

// Generic template constructor
template <typename T, typename... Args>
auto construct_unique(Args... args) -> std::unique_ptr<T> {
    return std::make_unique<T>(args...);
}

/////////////////////
// H1_FECollection //
/////////////////////

using mfem::H1_FECollection;

//////////
// Mesh //
//////////

using mfem::Mesh;

auto Mesh_Dimension(Mesh const& mesh) -> int {
    return mesh.Dimension();
}

auto Mesh_GetNE(Mesh const& mesh) -> int {
    return mesh.GetNE();
}

auto Mesh_UniformRefinement(Mesh& mesh, int ref_algo) -> void {
    mesh.UniformRefinement(ref_algo);
}
