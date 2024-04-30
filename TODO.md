# TODO

## Prioritized Entities

Required for writing (a minimal version of) [Example 1](https://github.com/mkovaxx/mfem/blob/69fbae732d5279c8d0f42c5430c4fd5656731d00/examples/ex1.cpp).

- [x] Mesh
- [x] Mesh::Dimension()
- [x] Mesh::GetNE()
- [x] Mesh::UniformRefinement()
- [x] Mesh::bdr_attributes
- [ ] Mesh::Print()
- [x] FiniteElementCollection
- [x] FiniteElementCollection::Name()
- [x] H1_FECollection
- [x] Mesh::GetNodes()
- [x] GridFunction
- [x] GridFunction::OwnFEC()
- [x] GridFunction::SetAll()
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
- [ ] BilinearForm::RecoverFEMSolution()
- [x] DiffusionIntegrator
- [x] OperatorHandle
- [x] Operator
- [x] Operator::Height()
- [x] Vector
- [ ] Vector::Save()
- [ ] GSSmoother
- [ ] PCG
