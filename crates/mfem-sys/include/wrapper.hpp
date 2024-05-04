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

//////////////
// ArrayInt //
//////////////

using ArrayInt = Array<int>;

auto ArrayInt_SetAll(ArrayInt& array, int value) -> void {
    array = value;
}

/////////////////////
// H1_FECollection //
/////////////////////

auto H1_FECollection_as_FEC(H1_FECollection const& h1_fec) -> FiniteElementCollection const& {
    return h1_fec;
}

//////////
// Mesh //
//////////

auto Mesh_GetNodes(Mesh const& mesh) -> GridFunction const& {
    auto ptr = mesh.GetNodes();
    if (!ptr) {
        throw mfem_exception("Mesh::GetNodes() == nullptr");
    }
    return *ptr;
}

auto Mesh_bdr_attributes(Mesh const& mesh) -> ArrayInt const& {
    return mesh.bdr_attributes;
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

auto FiniteElementSpace_GetEssentialTrueDofs(
    FiniteElementSpace const& fespace,
    ArrayInt const& bdr_attr_is_ess,
    ArrayInt& ess_tdof_list,
    int component
) -> void {
    // HACK(mkovaxx): This might come back to bite me...
    auto& mut_fespace = const_cast<FiniteElementSpace&>(fespace);
    mut_fespace.GetEssentialTrueDofs(bdr_attr_is_ess, ess_tdof_list, component);
}

//////////////////
// GridFunction //
//////////////////

auto GridFunction_as_Vector(GridFunction const& grid_func) -> Vector const& {
    return grid_func;
}

auto GridFunction_as_mut_Vector(GridFunction& grid_func) -> Vector& {
    return grid_func;
}

auto GridFunction_OwnFEC(GridFunction const& grid_func) -> FiniteElementCollection const& {
    auto ptr = const_cast<GridFunction&>(grid_func).OwnFEC();
    if (!ptr) {
        throw mfem_exception("GridFunction::OwnFEC() == nullptr");
    }
    return *ptr;
}

auto GridFunction_ctor_fes(FiniteElementSpace const& fespace) -> std::unique_ptr<GridFunction> {
    // HACK(mkovaxx): This might come back to bite me...
    auto& mut_fespace = const_cast<FiniteElementSpace&>(fespace);
    return std::make_unique<GridFunction>(&mut_fespace);
}

auto GridFunction_SetAll(GridFunction& grid_func, double value) {
    grid_func = value;
}

auto GridFunction_Save(GridFunction const& grid_func, std::string const& fname, int precision) {
    grid_func.Save(fname.c_str(), precision);
}

////////////////
// LinearForm //
////////////////

auto LinearForm_as_Vector(LinearForm const& lf) -> Vector const& {
    return lf;
}

auto LinearForm_ctor_fes(FiniteElementSpace const& fespace) -> std::unique_ptr<LinearForm> {
    // HACK(mkovaxx): This might come back to bite me...
    auto& mut_fespace = const_cast<FiniteElementSpace&>(fespace);
    return std::make_unique<LinearForm>(&mut_fespace);
}

auto LinearForm_AddDomainIntegrator(LinearForm& lf, std::unique_ptr<LinearFormIntegrator> lfi) {
    lf.AddDomainIntegrator(lfi.release());
}

/////////////////////////
// ConstantCoefficient //
/////////////////////////

auto ConstantCoefficient_as_Coeff(ConstantCoefficient const& coeff) -> Coefficient const& {
    return coeff;
}

////////////////////////
// DomainLFIntegrator //
////////////////////////

auto DomainLFIntegrator_ctor_ab(Coefficient const& coeff, int a, int b) -> std::unique_ptr<DomainLFIntegrator> {
    // HACK(mkovaxx): This might come back to bite me...
    auto& mut_coeff = const_cast<Coefficient&>(coeff);
    return std::make_unique<DomainLFIntegrator>(mut_coeff, a, b);
}

auto DomainLFIntegrator_as_LFI(DomainLFIntegrator const& domain_lfi) -> LinearFormIntegrator const& {
    return domain_lfi;
}

auto DomainLFIntegrator_into_LFI(std::unique_ptr<DomainLFIntegrator> domain_lfi) -> std::unique_ptr<LinearFormIntegrator> {
    return std::move(domain_lfi);
}

//////////////////
// BilinearForm //
//////////////////

auto BilinearForm_ctor_fes(FiniteElementSpace const& fespace) -> std::unique_ptr<BilinearForm> {
    // HACK(mkovaxx): This might come back to bite me...
    auto& mut_fespace = const_cast<FiniteElementSpace&>(fespace);
    return std::make_unique<BilinearForm>(&mut_fespace);
}

auto BilinearForm_AddDomainIntegrator(BilinearForm& bf, std::unique_ptr<BilinearFormIntegrator> bfi) {
    bf.AddDomainIntegrator(bfi.release());
}

auto BilinearForm_FormLinearSystem(
    BilinearForm const& a,
    ArrayInt const& ess_tdof_list,
    Vector const& x,
    Vector const& b,
    OperatorHandle& a_mat,
    Vector& x_vec,
    Vector& b_vec
) {
    // HACK(mkovaxx): This might come back to bite me...
    auto& mut_a = const_cast<BilinearForm&>(a);
    auto& mut_x = const_cast<Vector&>(x);
    auto& mut_b = const_cast<Vector&>(b);
    mut_a.FormLinearSystem(ess_tdof_list, mut_x, mut_b, a_mat, x_vec, b_vec);
}

/////////////////////////
// DiffusionIntegrator //
/////////////////////////

auto DiffusionIntegrator_ctor(Coefficient const& coeff) -> std::unique_ptr<DiffusionIntegrator> {
    // HACK(mkovaxx): This might come back to bite me...
    auto& mut_coeff = const_cast<Coefficient&>(coeff);
    return std::make_unique<DiffusionIntegrator>(mut_coeff);
}

auto DiffusionIntegrator_into_BFI(std::unique_ptr<DiffusionIntegrator> diffusion_bfi) -> std::unique_ptr<BilinearFormIntegrator> {
    return std::move(diffusion_bfi);
}

//////////////////
// OperatorType //
//////////////////

using OperatorType = Operator::Type;

////////////////////
// OperatorHandle //
////////////////////

auto OperatorHandle_as_ref(OperatorHandle const& handle) -> Operator const& {
    return *handle;
}

auto OperatorHandle_try_as_SparseMatrix(OperatorHandle const& handle) -> SparseMatrix const& {
    if (handle.Type() != OperatorType::MFEM_SPARSEMAT) {
        throw mfem_exception("OperatorHandle_try_as_SparseMatrix: wrong type");
    }
    return *handle.As<SparseMatrix>();
}

////////////////
// GSSmoother //
////////////////

auto GSSmoother_as_mut_Solver(GSSmoother& smoother) -> Solver& {
    return smoother;
}
