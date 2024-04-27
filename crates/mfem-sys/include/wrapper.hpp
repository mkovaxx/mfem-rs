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

auto H1_FECollection_as_fec(H1_FECollection const& h1_fec) -> FiniteElementCollection const* {
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

auto Mesh_GetNodes(Mesh const& mesh) -> GridFunction const* {
    return mesh.GetNodes();
}

////////////////////////
// FiniteElementSpace //
////////////////////////

using OrderingType = Ordering::Type;

auto FiniteElementSpace_ctor(Mesh& mesh, FiniteElementCollection const& fec, int vdim, OrderingType ordering) -> std::unique_ptr<FiniteElementSpace> {
    return std::make_unique<FiniteElementSpace>(&mesh, &fec, vdim, ordering);
}

//////////////////
// GridFunction //
//////////////////

auto GridFunction_OwnFEC(GridFunction const& grid_func) -> FiniteElementCollection const* {
    return const_cast<GridFunction&>(grid_func).OwnFEC();
}
