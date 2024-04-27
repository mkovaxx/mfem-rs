#include "rust/cxx.h"

#include "mfem.hpp"

// Generic template constructor
template <typename T, typename... Args>
auto construct_unique(Args... args) -> std::unique_ptr<T> {
    return std::make_unique<T>(args...);
}

using namespace mfem;

/////////////////////////////
// FiniteElementCollection //
/////////////////////////////

auto FiniteElementCollection_Name(FiniteElementCollection const& fec) -> char const* {
    return fec.Name();
}

/////////////////////
// H1_FECollection //
/////////////////////

auto H1_FECollection_as_fec(H1_FECollection& h1_fec) -> FiniteElementCollection* {
    return &h1_fec;
}

//////////
// Mesh //
//////////

auto Mesh_Dimension(Mesh const& mesh) -> int {
    return mesh.Dimension();
}

auto Mesh_GetNE(Mesh const& mesh) -> int {
    return mesh.GetNE();
}

auto Mesh_UniformRefinement(Mesh& mesh, int ref_algo) -> void {
    mesh.UniformRefinement(ref_algo);
}

auto Mesh_GetNodes(Mesh& mesh) -> GridFunction* {
    return mesh.GetNodes();
}

//////////////////
// GridFunction //
//////////////////

auto GridFunction_OwnFEC(GridFunction& grid_func) -> FiniteElementCollection* {
    return grid_func.OwnFEC();
}
