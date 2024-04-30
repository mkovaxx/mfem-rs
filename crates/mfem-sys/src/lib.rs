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

        fn Size(self: &ArrayInt) -> i32;
        fn Max(self: &ArrayInt) -> i32;
        fn ArrayInt_SetAll(array: Pin<&mut ArrayInt>, value: i32);

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

        fn H1_FECollection_as_fec(h1_fec: &H1_FECollection) -> &FiniteElementCollection;

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

        fn GridFunction_ctor_fes<'fes>(
            fespace: &'fes FiniteElementSpace,
        ) -> UniquePtr<GridFunction<'fes>>;

        fn GridFunction_OwnFEC<'a>(
            grid_func: &'a GridFunction,
        ) -> Result<&'a FiniteElementCollection>;
        fn GridFunction_SetAll(grid_func: Pin<&mut GridFunction>, value: f64);

        ////////////////
        // LinearForm //
        ////////////////

        type LinearForm<'a>;

        fn LinearForm_ctor_fes<'a>(fespace: &'a FiniteElementSpace) -> UniquePtr<LinearForm<'a>>;

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

        fn ConstantCoefficient_as_coeff(coeff: &ConstantCoefficient) -> &Coefficient;

        //////////////////////////
        // LinearFormIntegrator //
        //////////////////////////

        type LinearFormIntegrator<'a>;

        ////////////////////////
        // DomainLFIntegrator //
        ////////////////////////

        type DomainLFIntegrator<'a>;

        fn DomainLFIntegrator_ctor_ab<'a>(
            coeff: &'a Coefficient,
            a: i32,
            b: i32,
        ) -> UniquePtr<DomainLFIntegrator<'a>>;

        fn DomainLFIntegrator_into_lfi<'a>(
            domain_lfi: UniquePtr<DomainLFIntegrator<'a>>,
        ) -> UniquePtr<LinearFormIntegrator<'a>>;
    }
}
