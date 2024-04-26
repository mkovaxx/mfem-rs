#include "rust/cxx.h"

#include "mfem.hpp"

using namespace mfem;

// Generic template constructor
template <typename T, typename... Args>
auto construct_unique(Args... args) -> std::unique_ptr<T> {
    return std::unique_ptr<T>(new T(args...));
}

auto Mesh_GetNE(Mesh const& mesh) -> int {
    return mesh.GetNE();
}
