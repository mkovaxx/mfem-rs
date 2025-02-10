// Disable the surious warnings for the mfem header file.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wuninitialized"
#include "mfem.hpp"
#pragma GCC diagnostic pop

#ifndef RUST_AUTOCXX_H
#define RUST_AUTOCXX_H

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

#define SUBCLASS(A, B)                                          \
    /* Functions to cast A to a superclass B. */                \
    /* In C++ so errors can be caught by the type system */     \
    B* A##_as_mut_##B(A* x) {                                   \
        return static_cast<B*>(x);                              \
    }                                                           \
    const B* A##_as_##B(const A* x) {                           \
        return static_cast<const B*>(x);                        \
    }

    SUBCLASS(GridFunction, Vector)
    SUBCLASS(LinearForm, Vector)
    SUBCLASS(ConstantCoefficient, Coefficient)
    SUBCLASS(FunctionCoefficient, Coefficient)
    SUBCLASS(GridFunctionCoefficient, Coefficient)
    SUBCLASS(DomainLFIntegrator, DeltaLFIntegrator)
    SUBCLASS(DeltaLFIntegrator, LinearFormIntegrator)
    SUBCLASS(BilinearFormIntegrator, NonlinearFormIntegrator)
    SUBCLASS(DiffusionIntegrator, BilinearFormIntegrator)
    SUBCLASS(ConvectionIntegrator, BilinearFormIntegrator)
    SUBCLASS(Solver, Operator)
    SUBCLASS(MatrixInverse, Solver)
    SUBCLASS(SparseSmoother, MatrixInverse)
    SUBCLASS(GSSmoother, SparseSmoother)

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

#endif // RUST_AUTOCXX_H
