#[cxx::bridge]
pub mod ffi {
    #[repr(i32)]
    enum BasisType {
        Invalid = -1,
        /// Open type
        GaussLegendre = 0,
        /// Closed type
        GaussLobatto = 1,
        /// Bernstein polynomials
        Positive = 2,
        /// Nodes: x_i = (i+1)/(n+1), i=0,...,n-1
        OpenUniform = 3,
        /// Nodes: x_i = i/(n-1),     i=0,...,n-1
        ClosedUniform = 4,
        /// Nodes: x_i = (i+1/2)/n,   i=0,...,n-1
        OpenHalfUniform = 5,
        /// Serendipity basis (squares / cubes)
        Serendipity = 6,
        /// Closed GaussLegendre
        ClosedGL = 7,
        /// Integrated GLL indicator functions
        IntegratedGLL = 8,
        /// Keep track of maximum types to prevent hard-coding
        NumBasisTypes = 9,
    }

    #[repr(i32)]
    enum OrderingType {
        /// loop first over the nodes (inner loop) then over the vector dimension (outer loop)
        /// symbolically it can be represented as: XXX...,YYY...,ZZZ...
        byNODES,
        /// loop first over the vector dimension (inner loop) then over the nodes (outer loop)
        /// symbolically it can be represented as: XYZ,XYZ,XYZ,...
        byVDIM,
    }

    #[derive(Debug)]
    #[repr(i32)]
    enum OperatorType {
        /// ID for the base class Operator, i.e. any type.
        ANY_TYPE,
        /// ID for class SparseMatrix.
        MFEM_SPARSEMAT,
        /// ID for class HypreParMatrix.
        Hypre_ParCSR,
        /// ID for class PetscParMatrix, MATAIJ format.
        PETSC_MATAIJ,
        /// ID for class PetscParMatrix, MATIS format.
        PETSC_MATIS,
        /// ID for class PetscParMatrix, MATSHELL format.
        PETSC_MATSHELL,
        /// ID for class PetscParMatrix, MATNEST format.
        PETSC_MATNEST,
        /// ID for class PetscParMatrix, MATHYPRE format.
        PETSC_MATHYPRE,
        /// ID for class PetscParMatrix, unspecified format.
        PETSC_MATGENERIC,
        /// ID for class ComplexOperator.
        Complex_Operator,
        /// ID for class ComplexSparseMatrix.
        MFEM_ComplexSparseMat,
        /// ID for class ComplexHypreParMatrix.
        Complex_Hypre_ParCSR,
        /// ID for class ComplexDenseMatrix.
        Complex_DenseMat,
        /// ID for class BlockMatrix.
        MFEM_Block_Matrix,
        /// ID for the base class BlockOperator.
        MFEM_Block_Operator,
    }

    unsafe extern "C++" {
        // https://github.com/dtolnay/cxx/issues/280

        include!("mfem-sys/include/wrapper.hpp");

        //////////////
        // ArrayInt //
        //////////////

        type ArrayInt;

        #[cxx_name = "construct_unique"]
        fn ArrayInt_ctor() -> UniquePtr<ArrayInt>;

        #[cxx_name = "construct_unique"]
        fn ArrayInt_ctor_size(asize: i32) -> UniquePtr<ArrayInt>;

        fn GetData(self: &ArrayInt) -> *const i32;
        fn Size(self: &ArrayInt) -> i32;
        fn Max(self: &ArrayInt) -> i32;
        fn ArrayInt_SetAll(array: Pin<&mut ArrayInt>, value: i32);

        ////////////
        // Vector //
        ////////////

        type Vector;

        #[cxx_name = "construct_unique"]
        fn Vector_ctor() -> UniquePtr<Vector>;

        /////////////////////////////
        // FiniteElementCollection //
        /////////////////////////////

        type FiniteElementCollection;

        fn Name(self: &FiniteElementCollection) -> *const c_char;

        /////////////////////
        // H1_FECollection //
        /////////////////////

        type H1_FECollection;

        #[cxx_name = "construct_unique"]
        fn H1_FECollection_ctor(
            p: i32,
            dim: i32,
            btype: /*BasisType*/ i32,
        ) -> UniquePtr<H1_FECollection>;

        fn H1_FECollection_as_FEC(h1_fec: &H1_FECollection) -> &FiniteElementCollection;

        //////////
        // Mesh //
        //////////

        type Mesh;

        #[cxx_name = "construct_unique"]
        fn Mesh_ctor() -> UniquePtr<Mesh>;

        #[cxx_name = "construct_unique"]
        fn Mesh_ctor_file(
            filename: &CxxString,
            generate_edges: i32,
            refine: i32,
            fix_orientation: bool,
        ) -> UniquePtr<Mesh>;

        fn Dimension(self: &Mesh) -> i32;
        fn GetNE(self: &Mesh) -> i32;
        fn UniformRefinement(self: Pin<&mut Mesh>, ref_algo: i32);
        fn Mesh_GetNodes(mesh: &Mesh) -> Result<&GridFunction>;
        fn Mesh_bdr_attributes(mesh: &Mesh) -> &ArrayInt;
        fn Save(self: &Mesh, fname: &CxxString, precision: i32);

        ////////////////////////
        // FiniteElementSpace //
        ////////////////////////

        type OrderingType;

        type FiniteElementSpace<'mesh, 'fec>;

        fn FiniteElementSpace_ctor<'mesh, 'fec>(
            mesh: &'mesh Mesh,
            fec: &'fec FiniteElementCollection,
            vdim: i32,
            ordering: OrderingType,
        ) -> UniquePtr<FiniteElementSpace<'mesh, 'fec>>;

        fn GetTrueVSize(self: &FiniteElementSpace) -> i32;

        // This shim is needed because FiniteElementSpace::GetEssentialTrueDofs() isn't const
        fn FiniteElementSpace_GetEssentialTrueDofs(
            fespace: &FiniteElementSpace,
            bdr_attr_is_ess: &ArrayInt,
            ess_tdof_list: Pin<&mut ArrayInt>,
            component: i32,
        );

        //////////////////
        // GridFunction //
        //////////////////

        type GridFunction<'fes>;

        fn GridFunction_as_Vector<'a>(grid_func: &'a GridFunction) -> &'a Vector;

        fn GridFunction_as_mut_Vector<'a>(
            grid_func: Pin<&'a mut GridFunction>,
        ) -> Pin<&'a mut Vector>;

        fn GridFunction_ctor_fes<'fes>(
            fespace: &'fes FiniteElementSpace,
        ) -> UniquePtr<GridFunction<'fes>>;

        fn GridFunction_OwnFEC<'a>(
            grid_func: &'a GridFunction,
        ) -> Result<&'a FiniteElementCollection>;
        fn GridFunction_SetAll(grid_func: Pin<&mut GridFunction>, value: f64);

        fn GridFunction_Save(grid_func: &GridFunction, fname: &CxxString, precision: i32);

        ////////////////
        // LinearForm //
        ////////////////

        type LinearForm<'fes>;

        fn LinearForm_as_Vector<'a>(lf: &'a LinearForm) -> &'a Vector;

        fn LinearForm_ctor_fes<'fes>(
            fespace: &'fes FiniteElementSpace,
        ) -> UniquePtr<LinearForm<'fes>>;

        fn LinearForm_AddDomainIntegrator(
            lf: Pin<&mut LinearForm>,
            lfi: UniquePtr<LinearFormIntegrator>,
        );

        fn Assemble(self: Pin<&mut LinearForm>);

        /////////////////
        // Coefficient //
        /////////////////

        type Coefficient;

        /////////////////////////
        // ConstantCoefficient //
        /////////////////////////

        type ConstantCoefficient;

        #[cxx_name = "construct_unique"]
        fn ConstantCoefficient_ctor(c: f64) -> UniquePtr<ConstantCoefficient>;

        fn ConstantCoefficient_as_Coeff(coeff: &ConstantCoefficient) -> &Coefficient;

        //////////////////////////
        // LinearFormIntegrator //
        //////////////////////////

        type LinearFormIntegrator;

        ////////////////////////
        // DomainLFIntegrator //
        ////////////////////////

        type DomainLFIntegrator<'coeff>;

        fn DomainLFIntegrator_ctor_ab<'coeff>(
            coeff: &'coeff Coefficient,
            a: i32,
            b: i32,
        ) -> UniquePtr<DomainLFIntegrator<'coeff>>;

        fn DomainLFIntegrator_as_LFI<'coeff, 'a>(
            domain_lfi: &'a DomainLFIntegrator<'coeff>,
        ) -> &'a LinearFormIntegrator;

        fn DomainLFIntegrator_into_LFI<'coeff>(
            domain_lfi: UniquePtr<DomainLFIntegrator<'coeff>>,
        ) -> UniquePtr<LinearFormIntegrator>;

        //////////////////
        // BilinearForm //
        //////////////////

        type BilinearForm<'fes>;

        fn BilinearForm_ctor_fes<'fes>(
            fespace: &'fes FiniteElementSpace,
        ) -> UniquePtr<BilinearForm<'fes>>;

        fn BilinearForm_AddDomainIntegrator(
            bf: Pin<&mut BilinearForm>,
            bfi: UniquePtr<BilinearFormIntegrator>,
        );

        fn Assemble(self: Pin<&mut BilinearForm>, skip_zeros: i32);

        fn BilinearForm_FormLinearSystem(
            a: &BilinearForm,
            ess_tdof_list: &ArrayInt,
            x: &Vector,
            b: &Vector,
            a_mat: Pin<&mut OperatorHandle>,
            x_vec: Pin<&mut Vector>,
            b_vec: Pin<&mut Vector>,
        );

        fn RecoverFEMSolution(
            self: Pin<&mut BilinearForm>,
            x_vec: &Vector,
            b_vec: &Vector,
            x: Pin<&mut Vector>,
        );

        ////////////////////////////
        // BilinearFormIntegrator //
        ////////////////////////////

        type BilinearFormIntegrator<'a>;

        /////////////////////////
        // DiffusionIntegrator //
        /////////////////////////

        type DiffusionIntegrator<'coeff>;

        fn DiffusionIntegrator_ctor<'coeff>(
            coeff: &'coeff Coefficient,
        ) -> UniquePtr<DiffusionIntegrator<'coeff>>;

        fn DiffusionIntegrator_into_BFI<'coeff>(
            diffusion_int: UniquePtr<DiffusionIntegrator<'coeff>>,
        ) -> UniquePtr<BilinearFormIntegrator<'coeff>>;

        ////////////////////
        // OperatorHandle //
        ////////////////////

        type OperatorHandle;

        #[cxx_name = "construct_unique"]
        fn OperatorHandle_ctor() -> UniquePtr<OperatorHandle>;

        fn Type(self: &OperatorHandle) -> OperatorType;
        fn OperatorHandle_as_ref(handle: &OperatorHandle) -> &Operator;
        fn OperatorHandle_try_as_SparseMatrix(handle: &OperatorHandle) -> Result<&SparseMatrix>;

        //////////////////
        // OperatorType //
        //////////////////

        type OperatorType;

        //////////////
        // Operator //
        //////////////

        type Operator;

        fn Height(self: &Operator) -> i32;

        //////////////////
        // SparseMatrix //
        //////////////////

        type SparseMatrix;

        ////////////
        // Solver //
        ////////////

        type Solver;

        ////////////////
        // GSSmoother //
        ////////////////

        type GSSmoother<'mat>;

        #[cxx_name = "construct_unique"]
        fn GSSmoother_ctor<'mat>(
            a: &'mat SparseMatrix,
            t: i32,
            it: i32,
        ) -> UniquePtr<GSSmoother<'mat>>;

        fn GSSmoother_as_mut_Solver<'a>(smoother: Pin<&'a mut GSSmoother>) -> Pin<&'a mut Solver>;

        /////////
        // PCG //
        /////////

        fn PCG(
            a_mat: &Operator,
            solver: Pin<&mut Solver>,
            b_vec: &Vector,
            x_vec: Pin<&mut Vector>,
            print_iter: i32,
            max_num_iter: i32,
            rtolerance: f64,
            atolerance: f64,
        );
    }
}
