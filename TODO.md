# TODO

## Prioritized Entities

Required for writing (a minimal version of) [Example 1](https://github.com/mkovaxx/mfem/blob/69fbae732d5279c8d0f42c5430c4fd5656731d00/examples/ex1.cpp).

- [x] Mesh
- [x] Mesh::Dimension()
- [x] Mesh::GetNE()
- [x] Mesh::UniformRefinement()
- [x] Mesh::bdr_attributes
- [x] Mesh::Save()
- [x] FiniteElementCollection
- [x] FiniteElementCollection::Name()
- [x] H1_FECollection
- [x] Mesh::GetNodes()
- [x] GridFunction
- [x] GridFunction::OwnFEC()
- [x] GridFunction::SetAll()
- [x] GridFunction::Save()
- [x] FiniteElementSpace
- [x] FiniteElementSpace::GetTrueVSize()
- [x] FiniteElementSpace::GetEssentialTrueDofs()
- [x] Array<int>
- [x] Array<int>::Size()
- [x] Array<int>::Max()
- [x] Array<int>::SetAll()
- [x] LinearForm
- [x] LinearForm::AddDomainIntegrator()
- [x] LinearForm::Assemble()
- [x] ConstantCoefficient
- [x] DomainLFIntegrator
- [x] BilinearForm
- [x] BilinearForm::AddDomainIntegrator()
- [x] BilinearForm::Assemble()
- [x] BilinearForm::FormLinearSystem()
- [x] BilinearForm::RecoverFEMSolution()
- [x] DiffusionIntegrator
- [x] OperatorHandle
- [x] OperatorHandle::try_as_SparseMatrix()
- [x] Operator
- [x] Operator::Height()
- [x] Vector
- [x] GSSmoother
- [x] GSSmoother::as_mut_Solver()
- [x] Solver
- [x] PCG

## High-Level Wrapper

- [ ] Turn method-like wrapper functions into real methods
- [ ] Hide sharp bits such as UniquePtr, C/C++ strings, etc.
- [ ] Turn C++ base classes into traits to provide `.as_base<T>()` and `.as_mut_base<T>()`
- [ ] Unify `AsBase` and `IntoBase` traits

## Stretch Goals

- [ ] Get rid of `ArrayInt` and use `&[i32]` and `Vec<i32>` instead
- [ ] Make a `FunctionCoefficient` that can wrap a Rust `fn`-like thing
- [ ] `GridFunction::ProjectCoefficient()`
