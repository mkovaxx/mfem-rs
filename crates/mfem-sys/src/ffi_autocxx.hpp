// Disable the surious warnings for the mfem header file.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wuninitialized"
#include "mfem.hpp"
#pragma GCC diagnostic pop

#ifndef RUST_EXTRA_H
#define RUST_EXTRA_H

using namespace mfem;

// Easily extract what we have defined (autocxx will put the functions
// in a module with the same name).
namespace acxx {
    const bool MFEM_USE_EXCEPTIONS =
#ifdef MFEM_USE_EXCEPTIONS
        true;
#else
        false;
#endif

    const int NumBasisTypes = mfem::BasisType::NumBasisTypes;

    Array<int> const& Mesh_bdr_attributes(Mesh const& mesh) {
        return mesh.bdr_attributes;
    }

    // Immutable version
    FiniteElementCollection const* GridFunction_OwnFEC(GridFunction const& gf)
    {
        return const_cast<GridFunction&>(gf).OwnFEC();
    }

    std::unique_ptr<FiniteElementSpace> FES_new(
        Mesh & mesh,
        FiniteElementCollection const& fec,
        int vdim,
        Ordering::Type ordering)
    {
        return std::make_unique<FiniteElementSpace>(
            &mesh, &fec, vdim, ordering);
    }
}

#endif // RUST_EXTRA_H
