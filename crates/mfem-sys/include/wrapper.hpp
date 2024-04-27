#include "rust/cxx.h"

///////////////////////////////////////////
// Error Handler for Returning Result<T> //
///////////////////////////////////////////

class mfem_exception : public std::exception {
    std::string msg_;

public:
    explicit mfem_exception(char const* msg) : msg_(msg) {}
    auto what() const noexcept -> char const* override {
        return msg_.c_str();
    }
};

namespace rust {
namespace behavior {

template <typename Try, typename Fail>
static void trycatch(Try &&func, Fail &&fail) noexcept try {
  func();
} catch (const std::exception &e) {
  fail(e.what());
}

} // namespace behavior
} // namespace rust

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

auto H1_FECollection_as_fec(H1_FECollection const& h1_fec) -> FiniteElementCollection const& {
    return h1_fec;
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

auto Mesh_GetNodes(Mesh const& mesh) -> GridFunction const& {
    auto ptr = mesh.GetNodes();
    if (!ptr) {
        throw mfem_exception("Mesh::GetNodes() == nullptr");
    }
    return *ptr;
}

////////////////////////
// FiniteElementSpace //
////////////////////////

using OrderingType = Ordering::Type;

auto FiniteElementSpace_ctor(Mesh const& mesh, FiniteElementCollection const& fec, int vdim, OrderingType ordering) -> std::unique_ptr<FiniteElementSpace> {
    // HACK(mkovaxx): This might come back to bite me...
    auto& mut_mesh = const_cast<Mesh&>(mesh);
    return std::make_unique<FiniteElementSpace>(&mut_mesh, &fec, vdim, ordering);
}

//////////////////
// GridFunction //
//////////////////

auto GridFunction_OwnFEC(GridFunction const& grid_func) -> FiniteElementCollection const& {
    auto ptr = const_cast<GridFunction&>(grid_func).OwnFEC();
    if (!ptr) {
        throw mfem_exception("GridFunction::OwnFEC() == nullptr");
    }
    return *ptr;
}
