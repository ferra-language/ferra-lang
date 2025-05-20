# Ferra Data-Parallel Loops & GPU Support Design v0.1

> **Status:** Initial Draft - Module 3.5 (Steps 3.5.1 - 3.5.3)

## 1. Introduction and Goals

This document specifies Ferra's design for enabling data parallelism, primarily through a high-level `for_each` construct, and for supporting GPU computation via SPIR-V. The goal is to provide developers with idiomatic ways to express parallelizable computations and to allow Ferra to target both CPU SIMD (Single Instruction, Multiple Data) units and GPU hardware for accelerating these workloads.

Ferra aims to make data parallelism accessible and safe, leveraging its existing strengths in memory safety and type systems, while providing a clear path for offloading suitable computations to specialized hardware like GPUs.

*   **Purpose**:
    *   To define a unified language construct (e.g., a data-parallel `for_each`) for expressing computations that can operate independently on elements of a collection.
    *   To specify how such constructs can be transparently lowered to efficient CPU SIMD instructions, primarily by leveraging LLVM's auto-vectorization capabilities.
    *   To introduce a mechanism (e.g., a `#[gpu]` attribute) to mark specific functions or data-parallel loops for compilation to SPIR-V, enabling execution on compatible GPU hardware.
    *   To outline the necessary language features, compiler transformations, and runtime support for both CPU-based data parallelism and GPU offloading.

*   **Core Requirements (from `Steps.md` & `comprehensive_plan.md`)**:
    *   **Step 3.5.1: Design data-parallel `for_each` constructs**: This is the primary user-facing feature for expressing data parallelism.
    *   **Step 3.5.2: Specify lowering to CPU SIMD (LLVM auto-vectorization)**: The default execution target for data-parallel constructs on CPUs should aim for SIMD utilization. `Steps.md` (Item 5) explicitly notes this.
    *   **Step 3.5.3: Design SPIR-V generation for `#[gpu]` attribute**: Code marked with `#[gpu]` should be compilable to SPIR-V for execution on GPUs. `Steps.md` (Item 5) also specifies this.

*   **Goals for this Document**:
    1.  **Define `for_each` Construct**: Specify the syntax, semantics, type system interactions, and operational constraints of the data-parallel `for_each` construct.
    2.  **Specify `#[gpu]` Attribute**: Detail the syntax, application scope (e.g., functions, loops), and semantic implications of the `#[gpu]` attribute.
    3.  **CPU SIMD Lowering Strategy**: Outline how the `for_each` construct is translated into Ferra IR and subsequently LLVM IR to enable effective auto-vectorization for CPU targets.
    4.  **SPIR-V Generation Pipeline**: Describe the proposed compilation path for `#[gpu]` code to SPIR-V, including considerations for the language subset ("GPU-Ferra"), memory management, and interaction with SPIR-V tools/standards.
    5.  **Host-Device Interaction**: Address the mechanisms for invoking GPU kernels, managing data transfer between CPU (host) and GPU (device) memory, and synchronization.
    6.  **Identify Necessary Language and Standard Library Support**: Outline any new language features (beyond `for_each` and `#[gpu]`) or standard library modules required to support these capabilities (e.g., for GPU buffer management, kernel launching).

This specification aims to provide a foundational design for data parallelism and initial GPU support in Ferra, balancing ease of use for common parallel patterns with a clear path for targeting heterogeneous hardware.

## 2. Data-Parallel `for_each` Construct (Step 3.5.1)

The cornerstone of Ferra's user-facing data parallelism will be a `for_each` construct designed to express operations that can be applied independently to elements of a collection. This construct serves as the entry point for both CPU SIMD vectorization and GPU offloading (when combined with `#[gpu]`).

### 2.1. Rationale and Goals

*   **Need for a High-Level Construct**: Provide a way for developers to express "embarrassingly parallel" or "pleasantly parallel" computations without needing to manage threads, tasks, or low-level SIMD/GPU intrinsics directly for common cases.
*   **Ease of Use**: The syntax should be intuitive and similar to existing loop constructs in Ferra or other modern languages, making it easy to adopt.
*   **Clear Semantics**: The behavior of the parallel `for_each`, especially regarding data access, side effects, and iteration order (or lack thereof for true parallelism), must be well-defined.
*   **Targeting Collections/Iterators**: The construct should operate on common Ferra collection types (like `Vector<T>`, arrays) or types that can produce parallel iterators.
*   **Foundation for Optimization**: The construct should provide clear signals to the compiler that the enclosed operations are candidates for parallel execution (SIMD or GPU).

### 2.2. Syntax and Semantics

Two primary syntactic approaches are considered for the data-parallel `for_each`: a dedicated keyword/loop form or a method-based approach on parallel iterators.

*   **Conceptual EBNF for Method-Based Parallel Iteration (Preferred for v0.1)**:
    ```ebnf
    // Assumes `Expr` can resolve to a collection that has `par_iter` methods.
    // These are standard Ferra method calls, not new keywords.
    ParIterExpr   ::= Expr "." ("par_iter" | "par_iter_mut") "(" ")" 
                    ( "." ("enumerate" | "map" | "filter" /* etc. */) "(" (ClosureExpr)? ")" )* ; 
                    (* Chained parallel iterator adaptors *)

    ForEachCall   ::= ParIterExpr "." "for_each" "(" ClosureExpr ")" ;
    ClosureExpr   ::= (* Ferra's standard syntax for closures, e.g., `|arg| block_or_expr` *)
    ```
    *Note: The actual parsing of these constructs relies on Ferra's existing expression and method call grammar defined in **SYNTAX_GRAMMAR_V0.1.md** and corresponding AST nodes in **AST_SPECIFICATION.md** (e.g., `ExprKind::MethodCall`).*

*   **Option 1: Dedicated Loop Syntax (e.g., `par_for_each`)** (Less favored for initial simplicity)
    *   Conceptual Syntax:
        ```ferra
        par_for_each item in my_vector { // or `par_for_each (index, item) in my_vector.with_index()`
            // Loop body: process item
            // Restrictions apply here, e.g., limited side effects, no data dependencies between iterations.
        }
        ```
    *   This would require new keywords and grammar rules in `SYNTAX_GRAMMAR_V0.1.md`.

*   **Option 2: Method-Based on Parallel Iterators (Preferred for v0.1)**
    *   This approach leverages the iterator pattern, extending it for parallelism, similar to Rayon in Rust.
    *   **Conceptual Syntax**:
        ```ferra
        // Assuming `my_vector` is e.g., a `Vector<T>`
        // 1. Obtain a parallel iterator
        let par_iter = my_vector.par_iter(); // or some other method like .into_par_iter()

        // 2. Apply the for_each operation
        par_iter.for_each(|item_ref| { // Closure takes an immutable reference to an item
            // Process item_ref.
            // Example: perform a read-only computation
            let result = item_ref.calculate_something();
            // Side effects here need to be carefully managed (e.g., writing to a concurrent collection, or per-thread output then reduce)
            // For GPU, this closure body is what gets compiled into a kernel.
        });

        // For mutable access:
        my_vector.par_iter_mut().for_each(|item_mut_ref| { // Closure takes a mutable reference
            item_mut_ref.modify_in_place();
        });
        ```
    *   **Advantages**:
        *   Leverages existing iterator concepts, potentially more familiar.
        *   Extensible: Other parallel iterator adaptors (like `map`, `filter`, `reduce`) can be added later.
        *   Less new syntax; `par_iter()` and `for_each()` are methods.
    *   **Standard Library**: This would require additions to the standard library:
        *   Traits for parallel iterators (e.g., `ParallelIterator`).
        *   Implementations of these traits for core collection types (`Vector<T>`, arrays, slices).
        *   The `for_each` method on these parallel iterators.

*   **Semantics of the Loop Body**:
    *   **Independence of Iterations**: The core assumption is that each iteration of the `for_each` (i.e., the execution of the closure for each item) can be performed independently of other iterations without affecting their correctness.
    *   **Data Dependencies**: The compiler or runtime may not automatically handle data dependencies between iterations. If iteration `i` depends on the result of iteration `j`, this construct is likely inappropriate, or the dependency must be managed explicitly by the developer (e.g., through multiple `for_each` passes or by using different parallel algorithms like reductions).
    *   **Side Effects**:
        *   **CPU SIMD**: For CPU SIMD, side effects within the loop body (e.g., modifying shared state outside the iterated collection, I/O) can break vectorization or introduce non-determinism/races if not handled with extreme care (e.g., using atomics, thread-safe data structures). Generally discouraged for simple data-parallel loops.
        *   **GPU (`#[gpu]` context)**: Side effects are even more restricted. Typically, GPU kernels (derived from the `for_each` body) read from input buffers and write to output buffers. Arbitrary global state modification or I/O is usually not allowed directly from the kernel.
    *   **Iteration Order**: For parallel execution, the order in which items are processed is generally **not guaranteed**. Code within the `for_each` body should not rely on a specific sequence of execution relative to other items.
    *   **Return Value**: The `for_each` operation is typically side-effecting on the elements (if `par_iter_mut`) or used to produce results written to an output buffer/collection (often prepared before the loop). The `for_each` method itself usually returns `Unit`. Other parallel iterator methods like `map_reduce` could produce aggregate results.

*   **Closure Arguments**:
    *   The closure passed to `for_each` will typically take:
        *   An immutable reference (`&ItemType`) for `par_iter().for_each()`.
        *   A mutable reference (`&mut ItemType`) for `par_iter_mut().for_each()`.
        *   Optionally, an index if an indexed parallel iterator is used (e.g., `par_iter().enumerate().for_each(|(index, item_ref)| ...)`).

### 2.3. Interaction with Ferra's Type System and Ownership

Ferra's strong type system and ownership/borrowing rules are crucial for ensuring the safety of data-parallel operations, especially on the CPU.

*   **Data Access and Borrowing**:
    *   **`par_iter().for_each()` (Immutable borrows)**: The closure receives an immutable reference to each item. This allows multiple iterations to run concurrently on the CPU (or in parallel on SIMD lanes/GPU threads) reading from the collection without data races.
    *   **`par_iter_mut().for_each()` (Mutable borrows)**: The closure receives a mutable reference to each item.
        *   For true parallelism, the system must guarantee that each mutable reference is unique to its iteration (i.e., no two iterations try to mutate the same item simultaneously). This is naturally provided if iterating over distinct elements of a collection.
        *   The borrow checker will enforce that the closure does not capture mutable references to shared state in a way that would cause data races, unless that state is protected by appropriate synchronization primitives (though for simple `for_each`, the goal is to avoid needing explicit locks).
*   **Ensuring Data-Race Freedom (CPU)**:
    *   The combination of iterator splitting (how a parallel iterator divides work) and Ferra's borrow checker aims to prevent data races for CPU-based parallel execution.
    *   If the closure captures external data, normal borrowing rules apply. Immutable captures are generally fine. Mutable captures of shared data would require synchronization (e.g., `Mutex`, `Actor` state), which might make the loop body less suitable for simple `for_each` vectorization and push it towards actor-based parallelism instead.
*   **Type Constraints on Closure and Captured Variables**:
    *   The closure and any variables it captures must satisfy any necessary type constraints (e.g., `Send` and `Sync` if the parallel execution involves true OS threads, though for v0.1's deterministic actors and potential initial SIMD-focus, these might be different).
    *   For GPU execution (`#[gpu]` context), types of captured variables and data within the iterated collection must be "GPU-Ferra" compatible (see Section 4.2).

### 2.4. Examples

*   **Example 1: Simple Element-wise Vector Update (CPU SIMD target)**
    ```ferra
    fn scale_vector(vec: &mut Vector<f32>, factor: f32) {
        vec.par_iter_mut().for_each(|element| {
            *element = *element * factor;
        });
    }

    fn main() {
        var my_vec = Vector::from_slice([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        scale_vector(&mut my_vec, 2.5);
        // my_vec is now [2.5, 5.0, 7.5, 10.0, 12.5, 15.0, 17.5, 20.0]
        // The compiler would aim to vectorize the multiplication.
    }
    ```

*   **Example 2: Conditional Modification**
    ```ferra
    fn adjust_values(data: &mut Vector<i32>, threshold: i32) {
        data.par_iter_mut().for_each(|val| {
            if *val < threshold {
                *val = *val + 10;
            }
        });
    }
    ```
    *(Note: Conditional operations within the loop body can make auto-vectorization more challenging for CPU SIMD but are common in GPU kernels.)*

*   **Example 3: Read-only computation into an output buffer (GPU context later)**
    ```ferra
    fn compute_squares(input: &Vector<i32>, output: &mut Vector<i32>) {
        // Ensure output vector has the same length as input
        // For simplicity, assume output is pre-sized and mutable access is safe.
        // A more robust API might take input and return a new Vector.

        // This example focuses on the iteration. A real GPU version would need #[gpu]
        // and memory transfer mechanisms.
        input.par_iter().enumerate().for_each(|(index, item_ref)| {
            // Assuming direct write to output is possible and safe in this context.
            // For CPU, this might require `output` to be a concurrent collection or use indexing carefully.
            // For GPU, `output` would be a GPU buffer.
            if index < output.len() { // Bounds check
                 output[index] = (*item_ref) * (*item_ref);
            }
        });
    }
    ```
    *(This example highlights that writing to a separate output collection based on input often requires indexed iteration and careful consideration of how `output` is accessed and modified in a parallel context, which will be further detailed for GPU scenarios.)*

These examples illustrate the intended use of a method-based parallel `for_each`. The exact capabilities and safety guarantees, especially concerning side effects and shared state access from within the closure, will depend on the target (CPU SIMD vs. GPU) and the specifics of the parallel iterator implementation.

## 3. Lowering `for_each` to CPU SIMD (Step 3.5.2)

When a data-parallel `for_each` construct is not explicitly marked for GPU execution (i.e., no `#[gpu]` attribute is present, or the target platform does not support the GPU path), the Ferra compiler will aim to lower it for efficient execution on the CPU, primarily by generating code that is amenable to auto-vectorization into SIMD (Single Instruction, Multiple Data) instructions by the LLVM backend.

### 3.1. Target: Leverage LLVM's Auto-Vectorization

*   **Primary Strategy**: The main goal is to structure the generated LLVM IR in such a way that LLVM's existing, mature loop auto-vectorizer and SLP (Superword Level Parallelism) vectorizer can identify and exploit the data parallelism inherent in the `for_each` construct.
*   **Benefits**:
    *   Avoids the need for Ferra to implement its own complex vectorization logic.
    *   Leverages LLVM's sophisticated analyses and support for a wide range of SIMD instruction sets across different CPU architectures (SSE, AVX, NEON, etc.).
    *   (Ref: `BACKEND_LLVM_X86-64.md` Section 5.3 discusses semantic tag integration for vectorization hints).

### 3.2. AST to IR Transformation

The translation from the Ferra AST representation of a parallel `for_each` (e.g., a method call like `collection.par_iter().for_each(|item| {...})`) to Ferra IR needs to preserve the parallel semantics and structure it for vectorization.

*   **Representing Parallel Iteration in Ferra IR**:
    *   **Option 1: Specialized IR Instructions (More Complex)**: Introduce new Ferra IR opcodes specifically for parallel loops or iterators. This would make the parallel intent explicit in the IR.
    *   **Option 2: Lowering to Standard Loop Constructs with Metadata (Simpler for v0.1)**:
        *   Lower the `par_iter().for_each()` into a standard Ferra IR loop structure (e.g., similar to a `for` or `while` loop that iterates over the collection).
        *   The key is to attach metadata or semantic tags (see `IR_SEMANTIC_TAGS.md`) to this loop in the IR, indicating that it originated from a data-parallel `for_each` and is a candidate for vectorization. The existing `vectorize` tag (key 33) could be used or a more specific one like `data_parallel_loop`.
        *   The loop body would correspond to the closure passed to `for_each`.
*   **Ensuring Amenability to Auto-Vectorization**:
    *   **Simple Loop Structure**: The generated IR loop should be as simple as possible:
        *   Clear induction variable (if applicable after iterator desugaring).
        *   Computable loop bounds.
        *   No complex control flow or unanalyzable function calls within the innermost loop body if they would inhibit vectorization.
    *   **Absence of Data Dependencies**: The transformation must ensure (or rely on prior static analysis/type system guarantees) that there are no loop-carried data dependencies within the body that would prevent parallel execution of iterations. This is a core semantic requirement of the `for_each` construct itself.
    *   **Pointer Aliasing Information**: Providing LLVM with accurate aliasing information (e.g., if input and output buffers are known not to overlap) is crucial for vectorization. Ferra's borrow checker might provide some of this information, which could be translated into LLVM metadata (e.g., `noalias` attributes on pointers).

*   **Closure Handling**:
    *   The closure passed to `for_each` will be translated into a regular function or inlined into the loop body in the IR.
    *   Captured variables must be handled correctly, ensuring their values are available to each conceptual "lane" of the vectorized execution. If capturing by reference, aliasing rules are critical.

### 3.3. LLVM IR Generation

Once the Ferra IR represents the parallel loop with appropriate metadata, the backend's Ferra IR to LLVM IR conversion phase (see `BACKEND_LLVM_X86-64.md`) takes over.

*   **Mapping Loop Structures**: Translate the Ferra IR loop into an LLVM IR loop.
*   **Emitting Vectorization Hints**:
    *   If a `vectorize` or `data_parallel_loop` semantic tag is present on the Ferra IR loop, this should be translated into appropriate LLVM metadata to encourage the auto-vectorizer.
    *   This typically involves adding loop metadata like `llvm.loop.vectorize.enable`, `llvm.loop.vectorize.width`, etc., or using pragma-like constructs if Ferra supports them and they can influence LLVM.
    *   (Ref: `BACKEND_LLVM_X86-64.md` Section 3.8 and 5.3 already mention using semantic tags to inform LLVM attributes/metadata like `inlinehint` and `vectorize` loop metadata).
*   **Data Layout**: Ensure that data being processed (e.g., elements in a `Vector<T>`) is laid out in memory in a way that is conducive to SIMD operations (e.g., contiguous arrays of primitive types).

### 3.4. Limitations and Considerations

*   **Reliance on LLVM Auto-Vectorizer**: The success of CPU SIMD execution heavily depends on LLVM's ability to recognize and vectorize the generated loops.
    *   **When it Might Fail**: Auto-vectorization can be inhibited by:
        *   Complex control flow inside the loop body.
        *   Non-vectorizable function calls within the loop.
        *   Pointer aliasing ambiguities that LLVM cannot resolve.
        *   Data dependencies that were not apparent at the Ferra IR level.
        *   Non-contiguous memory access patterns.
    *   **Feedback**: The Ferra compiler should ideally provide feedback to the developer if a `for_each` loop intended for parallel execution could not be vectorized by LLVM, along with potential reasons (this might require parsing LLVM's optimization remarks).
*   **Data Layout (SoA vs. AoS)**:
    *   For optimal SIMD performance, Structure of Arrays (SoA) data layout is often preferred over Array of Structures (AoS) for collections of complex objects.
    *   Ferra's `for_each` will operate on the existing data layout. While the language might not enforce SoA, developers aiming for maximum SIMD performance should be aware of these considerations. Future language features or libraries might provide utilities for SoA transformations.
*   **Debugging Vectorized Code**:
    *   Debugging auto-vectorized code can be challenging, as the execution flow at the machine code level differs significantly from the scalar source.
    *   Standard debuggers will show behavior based on the original source, but understanding SIMD-specific performance issues might require lower-level profiling tools.
*   **Cost Model**: LLVM's auto-vectorizer uses a cost model to decide if vectorization is beneficial. Sometimes, for very small iteration counts or complex bodies, vectorization might not be applied or might even be detrimental. The `for_each` construct signals intent, but the final decision rests with LLVM for CPU SIMD.

The primary goal for v0.1 CPU SIMD support is to "do no harm" and structure the code in a way that gives LLVM the best chance to auto-vectorize common data-parallel patterns expressed via `for_each`. Explicit SIMD intrinsics are out of scope for this user-level feature.

## 4. GPU Support via `#[gpu]` Attribute and SPIR-V (Step 3.5.3)

Beyond CPU-based SIMD, Ferra aims to provide a pathway for offloading highly parallel computations to GPUs. This will be achieved by allowing developers to mark specific code for GPU execution using a `#[gpu]` attribute, which will then trigger a compilation pipeline targeting SPIR-V, an open standard intermediate representation for parallel compute and graphics.

### 4.1. The `#[gpu]` Attribute

The `#[gpu]` attribute is the primary mechanism for developers to indicate that a piece of Ferra code is intended to be compiled into a GPU kernel.

*   **Conceptual EBNF for `#[gpu]` Attribute Application**:
    ```ebnf
    GpuAttribute ::= "#[" "gpu" ( "(" AttributeArgumentList? ")" )? "]" ;
    // AttributeArgumentList would parse standard Ferra expressions, used for attribute parameters.
    // Example: #[gpu(target_env="vulkan1.2", workgroup_size_hint=[256,1,1])]

    AttributedFunctionDecl ::= GpuAttribute? AttributeListOpt ("pub")? ("unsafe")? ("async")? ("extern" AbiStringLiteral)? "fn" IDENTIFIER ParameterList ( "->" Type )? ( Block | ";" ) ;
    // This shows GpuAttribute as one of the attributes a function can have.
    // For application to loops, it would be an attribute on the loop statement/expression.
    ```
    *Note: The `#[gpu]` attribute itself is parsed using Ferra's general attribute syntax defined in **SYNTAX_GRAMMAR_V0.1.md** (Section 1.3.5), and its representation in the AST will be an `AttributeNode` as per **AST_SPECIFICATION.md** (Appendix).*

*   **Syntax and Application (TBD DPGPU-SYNTAX-1, DPGPU-ATTR-SCOPE-1)**:
    *   **Syntax**: `#[gpu]` or potentially `#[gpu(target_env="vulkan1.2", capabilities=["compute", "shader_float64"])]` if options are needed.
        *   The basic `#[gpu]` attribute will be defined in `SYNTAX_GRAMMAR_V0.1.md` (Section 1.3.5 for general attribute syntax). The specifics of its arguments, if any, will be detailed here.
        *   Its representation in the AST will follow `AST_SPECIFICATION.md` (Section 5 ItemKind::Attribute and Appendix for AttributeNode).
    *   **Application Scope**:
        1.  **On Functions**: Applying `#[gpu]` to a function (`async` or `sync`) would designate that entire function to be compiled into a GPU kernel.
            ```ferra
            #[gpu]
            fn my_gpu_kernel(input_buffer: &GpuBuffer<f32>, output_buffer: &mut GpuBuffer<f32>, factor: f32) {
                // GPU-compatible Ferra code here
                // Typically contains a data-parallel loop over elements
                par_for_each index in 0..input_buffer.len() { // Assuming this construct is used inside
                    output_buffer[index] = input_buffer[index] * factor * global_id_x(); // global_id_x() is a GPU intrinsic
                }
            }
            ```
        2.  **On Data-Parallel `for_each` Loops (Alternative/Complementary)**: Applying `#[gpu]` directly to a `par_for_each` loop could be another way to specify that this specific loop's body should become a GPU kernel.
            ```ferra
            fn process_on_cpu_then_gpu(data: &mut Vector<f32>) {
                // ... some CPU processing ...

                #[gpu] // Attribute applies to the following par_for_each
                data.par_iter_mut().for_each(|element| {
                    *element = (*element) * (*element); // Kernel logic
                });

                // ... more CPU processing ...
            }
            ```
        *   **Initial v0.1 Focus**: For simplicity, v0.1 might initially focus on `#[gpu]` applied to **entire functions**. The function body would then typically contain one or more data-parallel `for_each` loops that define the core parallel work. Applying `#[gpu]` directly to loops adds complexity in defining the implicit kernel signature and data capture. (This is part of TBD DPGPU-ATTR-SCOPE-1).

*   **Semantics**:
    *   The presence of `#[gpu]` signals to the Ferra compiler that the attributed function (or loop) must be compiled through a separate pipeline targeting a GPU backend (specifically SPIR-V for v0.1).
    *   It implies that the code within the `#[gpu]` scope must adhere to a restricted subset of the Ferra language ("GPU-Ferra," detailed in Section 4.2) suitable for GPU execution.
    *   It implies a different memory model and execution environment than standard CPU code.
    *   The compiler will verify that the code within the `#[gpu]` scope conforms to these restrictions. Violations will result in compile-time errors.

### 4.2. Language Subset for `#[gpu]` Code ("GPU-Ferra") (TBD DPGPU-SUBSET-1)

Code written within a `#[gpu]`-attributed function (or loop, if supported) must adhere to a restricted subset of the Ferra language, informally termed "GPU-Ferra." This is necessary because GPU architectures and execution models differ significantly from CPUs. The goal is to define a subset that is expressive enough for common parallel computations but can be reliably and efficiently compiled to SPIR-V and executed on typical GPU hardware.

The precise definition of this subset is a critical TBD (DPGPU-SUBSET-1). Below are the anticipated areas of restriction and consideration:

*   **Allowed Data Types**:
    *   **Primitives**: Integer types (`i32`, `u32`, `i64`, `u64` - support for 64-bit integers depends on target GPU/SPIR-V capabilities), floating-point types (`f32`, `f64` - `f64` support also depends on target capabilities), `Bool`.
    *   **Simple Structs/Data Classes**: Ferra `data` classes composed *only* of other allowed GPU-Ferra primitive types or fixed-size arrays of these primitives. These must have a layout compatible with GPU memory structures (e.g., implicitly or explicitly `#[repr(C)]` or `#[repr(SPIRV)]` if such an attribute is introduced).
    *   **Fixed-Size Arrays**: `[T; N]` where `T` is an allowed GPU-Ferra type and `N` is a compile-time constant. These map well to arrays in GPU shader languages.
    *   **Vector Types (Intrinsic-like)**: Potentially, built-in vector types like `float2`, `float4`, `int2`, `int4` common in GPU programming. These might be provided by a `core::gpu` module and map directly to SPIR-V vector types.
    *   **Pointers/References**: Raw pointers or restricted forms of references to data in GPU memory (global, group-shared, or private memory, see below). Ferra's ownership/borrowing system might have a simplified or adapted interpretation within GPU kernels.

*   **Disallowed Data Types (Examples)**:
    *   **Dynamically Sized Collections**: Ferra `Vector<T>` (as a type directly manipulated with `push`/`pop` inside the kernel), `String`, `Map<K,V>`. Dynamic memory allocation is generally not supported or is highly restricted in GPU kernels. Data is typically pre-allocated in buffers by the host.
    *   **Complex User-Defined Types**: Data classes with heap-allocated fields, closures capturing complex environments (unless translatable to simple data), traits/dynamic dispatch (unless resolvable at compile time for the GPU kernel).
    *   **Most Standard Library Types**: Types relying on OS services, dynamic allocation, or complex CPU-centric logic.

*   **Disallowed Operations and Language Features**:
    *   **Dynamic Memory Allocation**: Calls to `ferra_alloc`, `ferra_free`, or operations that implicitly allocate on a general-purpose heap (e.g., growing a `Vector`).
    *   **Standard I/O**: `println!`, file operations.
    *   **FFI Calls**: Calling arbitrary external C functions.
    *   **Complex Control Flow**: Some complex control flow that is hard to map to efficient GPU execution (e.g., arbitrary recursion, deep exception handling if Ferra has it). Simple loops and branches are usually fine.
    *   **String Manipulation**: Rich string manipulation routines.
    *   **Threading/Actor Spawning**: Creating new CPU threads or Ferra actors from within GPU code.
    *   **Blocking System Calls**.
    *   **Reflection or Runtime Type Information (RTTI)**.

*   **Allowed Control Flow**:
    *   `if/else` statements.
    *   `for` loops with clear bounds (e.g., iterating over a range, or a `par_for_each` over a GPU buffer).
    *   `while`, `loop` with restrictions to ensure they can be mapped to GPU control flow (e.g., ensuring termination, avoiding excessive divergence).
    *   `return`, `break`, `continue` within their valid scopes.

*   **Function Call Limitations**:
    *   Calls are generally restricted to:
        *   Other `#[gpu]`-marked Ferra functions (which must also adhere to the GPU-Ferra subset). These can be inlined by the SPIR-V compiler.
        *   GPU-specific intrinsic functions provided by a `core::gpu` module (e.g., for accessing thread IDs, barrier synchronization, texture sampling, atomic operations).
        *   Pure mathematical functions from a restricted part of the standard library that can be compiled for GPU.

*   **Memory Model (GPU Perspective) (TBD DPGPU-MEMMODEL-1)**:
    GPU kernels operate with distinct memory spaces, and GPU-Ferra code must be written with awareness of these if direct control is exposed. The `core::gpu` module will provide abstractions for buffer management and data transfers (see Section 4.4 and 4.5).

    | GPU Memory Space      | Conceptual Description                                                                 | Managed Via (Host API)          | Accessible From Kernel Via...             |
    |-----------------------|----------------------------------------------------------------------------------------|---------------------------------|-------------------------------------------|
    | **Global Memory**     | Main device memory. Large buffers for kernel input/output. Persists across kernel calls. | `GpuBuffer<T>` allocation/transfer | Pointers/handles to `GpuBuffer<T>` arguments | 
    | **Group-Shared Memory** (Local Memory) | Fast on-chip memory shared by threads within a single workgroup/thread block. Used for cooperative computation, caching. | Kernel-side declaration (e.g., `let shared_data: [f32; 256] = workgroup_shared_array();`) | Special local/shared pointers or types  |
    | **Private Memory**    | Registers and thread-local stack memory. Private to each GPU thread.                     | Implicit (compiler managed)     | Regular local variables                   |
    | **Constant Memory**   | Read-only memory, often cached, for parameters uniform across all threads.             | Special buffer type or uniform upload | Uniform variables or constant buffers     |

    *   **Memory Access**:
        *   Access to global and group-shared memory will likely be through specialized pointer types or buffer types provided by a `core::gpu` module for use within `#[gpu]` kernels.
        *   Ferra's ownership/borrowing system needs a clear interpretation in this context. While it can prevent data races within a single thread's private data, shared memory access requires explicit GPU synchronization primitives (barriers).
    *   **Data Layout**: Performance on GPUs is highly sensitive to memory access patterns (coalescing). While GPU-Ferra might not enforce specific layouts like SoA for user types initially, documentation should highlight these considerations.

The GPU-Ferra subset will necessarily be more restrictive than full Ferra, reflecting the specialized nature of GPU programming. The compiler must statically enforce these restrictions for `#[gpu]` code.

### 4.3. SPIR-V Generation Pipeline

Once Ferra code within a `#[gpu]` scope has been parsed and semantically analyzed (and validated against the GPU-Ferra subset), it needs to be compiled into SPIR-V.

*   **Input**:
    *   The Ferra IR corresponding to the `#[gpu]`-marked function or loop body.
    *   Information about the kernel signature (input/output buffers, uniform parameters) derived from the function signature or the `for_each` context.

*   **Translation Process (TBD DPGPU-SPIRV-TOOL-1)**: Several strategies exist for generating SPIR-V:
    1.  **Ferra IR -> LLVM IR -> LLVM-SPIR-V Backend**:
        *   **Pros**: Leverages LLVM's optimization capabilities. LLVM has an official (or well-supported unofficial) SPIR-V backend.
        *   **Cons**: Adds LLVM as a dependency for GPU compilation. The LLVM-SPIR-V backend needs to be mature and support the required features. Mapping Ferra IR semantics perfectly to LLVM IR in a way that then translates well to SPIR-V compute shaders requires care.
        *   (Ref: `BACKEND_LLVM_X86-64.md` details existing LLVM integration).
    2.  **Ferra IR -> Direct SPIR-V Generation**:
        *   **Pros**: More direct control over SPIR-V generation. Avoids LLVM dependency for this path if not otherwise needed for GPU.
        *   **Cons**: Requires implementing a significant compiler backend to translate Ferra IR constructs (loops, memory access, function calls) into SPIR-V's specific instruction set and module structure. This is a substantial effort.
    3.  **Ferra IR -> High-Level Shading Language (HLSL/GLSL/WGSL) -> SPIR-V**:
        *   **Pros**: Could leverage existing mature compilers for HLSL/GLSL/WGSL to SPIR-V (e.g., `glslangValidator`, `dxc`, `naga`).
        *   **Cons**: Introduces an intermediate language, potentially losing some Ferra-specific semantic information or control. Requires a Ferra IR to HLSL/GLSL/WGSL translator.
    *   **Initial Leaning for v0.1**:
        *   Given Ferra's existing plan for an LLVM backend for CPUs, **Option 1 (via LLVM-SPIR-V backend)** is often a pragmatic starting point if LLVM is already a core compiler dependency and its SPIR-V backend is sufficiently capable for compute shaders. This would require ensuring our Ferra IR for `#[gpu]` code translates to LLVM IR that the SPIR-V backend can consume effectively for compute.
        *   If direct SPIR-V generation (Option 2) is chosen, libraries like `rspirv` (Rust) can aid in constructing SPIR-V modules programmatically.

*   **Choice of Tools/Libraries (If not purely LLVM)**:
    *   If using Option 2 or 3, libraries/tools would be needed:
        *   For SPIR-V construction (e.g., `rspirv` in Rust).
        *   For validation (e.g., `spirv-val` from SPIRV-Tools).
        *   For optimization (e.g., `spirv-opt` from SPIRV-Tools).

*   **Target SPIR-V Version and Capabilities**:
    *   **Version**: Target a reasonably modern and widely supported SPIR-V version (e.g., SPIR-V 1.3 - 1.5).
    *   **Execution Model**: Primarily `GLCompute` for compute kernels.
    *   **Capabilities**: Declare necessary SPIR-V capabilities based on the features used by the GPU-Ferra subset (e.g., `Shader`, `Float64` if f64 is supported, `Int64` if i64 is supported, kernel execution modes, addressing models).
    *   **Target Environment**: Initially, likely Vulkan compute environment. OpenCL via SPIR-V is also a possibility.

The SPIR-V generation pipeline must produce valid, reasonably optimized SPIR-V modules that accurately reflect the semantics of the GPU-Ferra code.

### 4.4. Host-Device Interaction (TBD DPGPU-MEMMODEL-1)

Effective GPU computing requires careful management of interactions between the host (CPU) and the device (GPU), particularly concerning kernel launching and data movement. The specifics of this interaction model, including memory management APIs, are a key TBD (DPGPU-MEMMODEL-1).

*   **Kernel Invocation**:
    *   **Mechanism**: Ferra code running on the CPU (host) will need a way to launch a `#[gpu]`-compiled kernel (SPIR-V module) on the GPU.
    *   **Standard Library API**: This will likely be through functions in a `core::gpu` or `std::gpu` module.
        ```ferra
        // Conceptual API
        // Assuming my_gpu_kernel is a #[gpu] fn:
        // fn my_gpu_kernel(input: &GpuBuffer<f32>, output: &mut GpuBuffer<f32>, len: u32);

        // Host code:
        // let gpu_context = gpu::Context::acquire_default_device()?; // Acquire GPU device context
        // let compiled_kernel = gpu_context.compile_kernel(my_gpu_kernel_identifier_or_path)?; // Or compiler provides this

        // Prepare GpuBuffer instances for input and output (see Data Management)
        // let input_gpu_buf = gpu_context.create_buffer_from_host_data(&input_cpu_vec, BufferUsage::ReadOnly)?;
        // let output_gpu_buf = gpu_context.create_buffer(output_cpu_vec.len(), BufferUsage::WriteOnly)?;
        
        // Define execution grid (workgroups, workgroup size)
        // let dispatch_dims = GpuDispatchDimensions { global_x: input_cpu_vec.len(), local_x: 256 }; // Example

        // Launch the kernel
        // gpu_context.launch_kernel(
        //    compiled_kernel, 
        //    dispatch_dims, 
        //    &[input_gpu_buf.as_argument(), output_gpu_buf.as_argument(), input_cpu_vec.len() as u32] // Arguments matching kernel signature
        // )?;
        ```
    *   **Kernel Arguments**: The host must be able to pass arguments to the GPU kernel, typically pointers to GPU buffers (see below) and any small uniform values (like `factor` or `len` in previous examples).
    *   **Dispatch Grid**: The host specifies the execution grid (e.g., number of workgroups and workgroup size) for the kernel launch.

    *   **Data Management**:
    *   **Separate Memory Spaces**: CPUs and discrete GPUs typically have separate physical memory spaces. Data must be explicitly transferred.
    *   **GPU Buffers (`GpuBuffer<T>`)**: A standard library type, e.g., `GpuBuffer<T>`, will represent a region of memory allocated on the GPU device.
        *   `T` must be a GPU-Ferra compatible type.
    *   **Allocation**: APIs to allocate GPU buffers:
        *   `gpu::Context::create_buffer<T>(element_count: usize, usage: BufferUsage) -> Result<GpuBuffer<T>, GpuError>`
        *   `BufferUsage` enum: e.g., `ReadOnly`, `WriteOnly`, `ReadWrite`, `HostToDevice`, `DeviceToHost`.
    *   **Data Transfer**: APIs to copy data between host memory (e.g., Ferra `Vector<T>`) and `GpuBuffer<T>`:
        *   `gpu_buffer.write_from_host(host_data: &[T]) -> Result<(), GpuError>`
        *   `gpu_buffer.read_to_host(host_buffer: &mut [T]) -> Result<(), GpuError>`
        *   Or, `gpu_context.create_buffer_from_host_data(host_data: &[T], usage: BufferUsage) -> Result<GpuBuffer<T>, GpuError>`
    *   **Mapping/Unmapping (Advanced)**: For integrated GPUs or systems with unified memory, more advanced mapping capabilities might be exposed to provide zero-copy access, but initial v0.1 will likely focus on explicit transfers for broader compatibility.
    *   **Ownership and Synchronization**:
        *   `GpuBuffer<T>` instances in Ferra host code act as handles to device memory. Their lifetimes must be managed.
        *   Data transfers are typically asynchronous operations from the host's perspective. Synchronization is needed to ensure data is available before use or that GPU work is complete before reading results (see below).
        *   Ferra's ownership system can help manage the lifetime of `GpuBuffer` handles on the host, potentially using `Drop` to ensure device memory is deallocated when the handle goes out of scope (if not explicitly freed).

*   **Synchronization**:
    *   **Host Waiting for Device**: The host needs a way to wait for GPU kernel execution to complete before accessing output data.
        *   `gpu_context.wait_for_idle()` or a per-kernel completion future/handle.
    *   **Device-Side Synchronization (within kernels)**:
        *   **Barriers**: For synchronizing threads within a workgroup (`workgroup_barrier()`).
        *   **Memory Fences**: To ensure correct ordering of memory operations to shared or global memory.
        *   These would be exposed as intrinsics or functions in the `core::gpu` module for use within `#[gpu]` code.

The API for host-device interaction needs to be ergonomic while exposing necessary control for performance. It will likely be inspired by existing GPU compute APIs like Vulkan, Metal, or WebGPU, but abstracted for Ferra.

### 4.5. Standard Library Support for GPU Operations

To support GPU programming effectively, Ferra's standard library will need a dedicated module (e.g., `core::gpu` or `std::gpu`).

*   **GPU Context and Device Management**:
    *   APIs to query available GPU devices.
    *   APIs to create a GPU context for a selected device.
*   **Kernel Compilation and Launching**:
    *   Functions to load/compile `#[gpu]` functions (SPIR-V modules) if not handled entirely by the main compiler.
    *   Functions to launch kernels, specifying dispatch dimensions and arguments (as seen in 4.4).
*   **Buffer Management APIs**:
    *   The `GpuBuffer<T>` type and its associated methods for allocation, deallocation, data transfer (`write_from_host`, `read_to_host`).
*   **GPU Intrinsics**:
    *   Functions callable only from within `#[gpu]` code to access GPU-specific information or operations:
        *   Work-item/thread IDs (e.g., `global_id_x()`, `local_id_y()`, `workgroup_id_z()`).
        *   Workgroup dimensions (e.g., `workgroup_size_x()`).
        *   Synchronization primitives (e.g., `workgroup_barrier()`, memory fences).
        *   Atomic operations for shared memory (e.g., `atomic_add_u32(...)`).
        *   Specialized math functions (e.g., `fast_normalize()`, `dot_product()`) that map efficiently to GPU hardware.
*   **GPU-Specific Data Types**:
    *   Vector types (e.g., `float2`, `float3`, `float4`, `int2`, `int4`, etc.) and matrix types if desired, with associated operations. These should map to efficient SPIR-V types.
*   **Error Handling Types**:
    *   `GpuError` enum/struct for errors related to GPU operations (e.g., device unavailable, kernel launch failure, memory allocation failure, data transfer error).

This module will abstract the low-level details of interacting with the underlying GPU driver or compute API (e.g., Vulkan).

### 4.6. Error Handling for GPU Code

Errors can occur at various stages of GPU programming: kernel compilation, kernel launch, data transfer, or during kernel execution on the device.

*   **Host-Side Errors**:
    *   Errors during device discovery, context creation, buffer allocation, data transfer, or kernel launching will be reported to the host Ferra code via `Result<T, GpuError>` from the `core::gpu` API functions.
    *   `GpuError` will be a Ferra error type detailing the nature of the failure.
*   **Device-Side (Kernel) Errors (TBD DPGPU-ERROR-1)**:
    *   Handling errors that occur *during* the execution of a kernel on the GPU is more complex.
    *   **No Traditional Exceptions**: SPIR-V and most GPU execution models do not support traditional exception handling that unwinds back to the host.
    *   **Mechanisms**:
        1.  **Error Codes/Flags**: Kernels can write error codes or flags to a designated output buffer. The host code reads this buffer after kernel completion to check for errors.
        2.  **Device-Side Assertions/Traps**: Some GPU environments allow kernels to trap or halt on error, which might be detectable by the host API.
        3.  **Debug Layers/Validation**: GPU APIs (like Vulkan) have validation layers that can catch many errors during development (e.g., out-of-bounds access, invalid API usage). Ferra tooling should encourage or enable these for debug builds.
    *   For v0.1, device-side error reporting might be limited, with a focus on making kernel launches and data transfers robust. More sophisticated device-side error handling is a TBD.

*   **Debugging `#[gpu]` Code**:
    *   Debugging GPU kernels is notoriously difficult. Initial support will likely rely on:
        *   Careful "printf-style" debugging by writing intermediate values to output buffers.
        *   Using vendor-provided GPU debugging tools (e.g., Nvidia Nsight, AMD RGP, Intel Graphics Performance Analyzers) on the generated SPIR-V or the driver's representation, if possible.
        *   The restrictions of the GPU-Ferra subset should help reduce the classes of bugs possible within kernel code.

### 4.7. Semantic Tag for `#[gpu]`

To enable the compiler backend to differentiate code intended for GPU execution, a specific semantic tag will be used.

*   **Tag Name**: `gpu_kernel` (or `spirv_target`, `target_gpu_compute`)
*   **Application**: This tag will be applied by the compiler to Ferra IR functions or loop structures that originate from Ferra source code annotated with `#[gpu]`.
*   **Purpose**:
    *   Signals to the IR-to-Backend lowering phase that this piece of IR should be routed to the SPIR-V generation pipeline instead of the standard CPU backend (e.g., LLVM).
    *   Allows validation passes to check this IR against the GPU-Ferra language subset rules.
*   **Storage**: This tag will be defined in **IR_SEMANTIC_TAGS.md** (e.g., in a section for backend/target-specific tags) with an assigned numeric key (TBD DPGPU-IR-REP-1, related to TBD item in **IR_SEMANTIC_TAGS.md** for assigning a key to `target::gpu_kernel`). The value could be boolean `true`, or a CBOR map containing target GPU features if the `#[gpu(...)]` attribute takes parameters.
*   **Cross-Reference**: The `#[gpu]` attribute documentation in `SYNTAX_GRAMMAR_V0.1.md` and `AST_SPECIFICATION.md` should note its translation to this IR semantic tag.

This tag provides the crucial link between the source-level attribute and the backend compilation choice.

## 5. Interaction with Concurrency Model

Ferra's primary concurrency model is based on deterministic actors (see `CONCURRENCY_MODEL.md`). The introduction of data-parallel `for_each` constructs and GPU offloading needs to interact coherently with this model.

*   **Data-Parallel `for_each` (CPU SIMD) within Actors**:
    *   An actor, within its message handler (`async fn handle_...`), can certainly use the data-parallel `for_each` construct on collections it owns or has mutable access to.
    *   Since the actor model processes messages serially for a given actor instance (for v0.1), a `for_each` loop targeting CPU SIMD within a handler will execute as part of that message processing turn.
    *   The parallelism achieved is at the data level (SIMD lanes) within the actor's sequential message processing flow, not by spawning new concurrent tasks or actors for each iteration.
    *   This is a natural fit: actors can use `for_each` to speed up data-intensive parts of their state updates or computations.

*   **GPU Offloading (`#[gpu]`) from Actors**:
    *   An actor should be able to initiate GPU computations.
    *   **Asynchronous Offload**: Launching a GPU kernel and waiting for its result is inherently an asynchronous operation from the host's (CPU/actor's) perspective.
        *   The `core::gpu` API for launching kernels (Section 4.4) should be `async`. An actor would `await` the completion of the GPU task.
        *   `await gpu_context.launch_kernel(...);`
        *   `await gpu_output_buffer.read_to_host(...);`
    *   While `await`ing a GPU operation, the actor yields control to the Ferra scheduler, allowing other actors (or other messages for the same actor, if re-entrancy is ever supported) to run. This is consistent with the `async`/`await` semantics in `CONCURRENCY_MODEL.md`.
    *   **Data Ownership and Transfer**:
        *   Data to be processed by the GPU must be transferred from the actor's state (CPU memory) to GPU buffers.
        *   Results must be transferred back.
        *   The `GpuBuffer<T>` handles would be owned by the actor's state or a specific task within the actor. Ferra's ownership and borrowing rules would apply to these handles on the host side.
        *   Care must be taken to ensure that data passed to the GPU is not mutated on the CPU side while the GPU is potentially reading from it (if using shared memory models, though explicit copy is safer and more common for discrete GPUs).

*   **Synchronization between Actor Tasks and GPU Computations**:
    *   The `await` mechanism on GPU operations (kernel launch completion, data transfer completion) provides the primary synchronization point.
    *   If an actor needs to perform other work while a GPU computation is in flight and then react to its completion, it could potentially use a mechanism where the GPU completion signals an event or sends a message back to the originating actor or another designated actor (this is more advanced than simple `await`). For v0.1, direct `await` is the primary model.

*   **No Direct Actor Spawning on GPU**:
    *   The Ferra actor model is a CPU-level concurrency construct. `#[gpu]` code compiles to kernels that run on the GPU's distinct parallel architecture.
    *   It's not envisioned that Ferra actors themselves would be spawned or run directly on the GPU. Instead, actors on the CPU offload specific data-parallel tasks to the GPU.

The `for_each` construct provides a tool for fine-grained data parallelism within an actor's processing turn (CPU SIMD) or for offloading larger parallel computations to the GPU in a non-blocking way from the actor's perspective.

## 6. Tooling and Developer Experience

Effective tooling and a good developer experience are crucial for making data parallelism and GPU programming accessible and productive in Ferra.

*   **Compiler Flags**:
    *   `--target-cpu-features=[+simd,+avx2,-avx512]` (or similar): Flags to control CPU-specific SIMD feature generation if more fine-grained control than default auto-vectorization is needed (usually LLVM handles this based on target CPU).
    *   `--enable-gpu-spirv` / `--target-gpu=<gpu_arch_profile>`: Flags to enable the SPIR-V compilation pipeline for `#[gpu]` code and potentially specify a target GPU architecture profile (e.g., for selecting SPIR-V capabilities or optimizations).
    *   `--gpu-optimization-level=<level>`: Specific optimization level for SPIR-V generation.
    *   `--ferra-diag=vectorization` / `--ferra-diag=gpu`: Flags to get detailed diagnostic information from the compiler about SIMD vectorization attempts (successes, failures, reasons) and GPU compilation.

*   **Example CLI Usage (Conceptual)**:
    ```bash
    # Build with GPU support enabled, targeting a generic Vulkan environment
    ferra build --enable-gpu-spirv --target-gpu=vulkan_generic

    # Build with verbose diagnostics for CPU vectorization
    ferra build --ferra-diag=vectorization

    # Build a specific file with GPU compilation and output SPIR-V (illustrative)
    # (Actual command for emitting SPIR-V directly might differ or be part of build artifacts)
    ferra compile my_gpu_kernels.ferra --emit=spirv --output=my_kernels.spv --enable-gpu-spirv 
    ```

*   **Debugging Support**:
    *   **CPU SIMD**: Debugging auto-vectorized code can be challenging. Debuggers will typically step through the original scalar loop. Understanding SIMD performance might require CPU profilers that can show vector instruction usage.
    *   **GPU SPIR-V Code**:
        *   This is significantly more complex. Standard CPU debuggers cannot typically step into GPU kernels directly.
        *   **Initial v0.1**: Debugging will likely rely on:
            *   "Printf-style" debugging by writing intermediate values from the kernel to output buffers that are then copied back to the host for inspection.
            *   Careful unit testing of `#[gpu]` functions with known inputs and outputs.
            *   Using external vendor-specific GPU debugging tools (e.g., Nvidia Nsight, AMD CodeXL/RGP, Intel Graphics Performance Analyzers) if they can consume SPIR-V or the driver's compiled form. This requires developers to use those tools outside the core Ferra environment.
        *   **Future**: Explore if Ferra can integrate with or provide wrappers for GPU debugging APIs or standards if they evolve.
        *   The restrictive "GPU-Ferra" subset (Section 4.2) should help reduce the likelihood of certain types of bugs in kernel code.

*   **Compiler Feedback and Diagnostics**:
    *   **Vectorization Reports (CPU SIMD)**: The compiler should (perhaps under a verbose flag) report on which `for_each` loops were successfully vectorized for CPU SIMD and which were not, with reasons for failure (e.g., "loop contains non-vectorizable function call," "data dependency prevents vectorization"). This reporting should align with the principles and structured format of **DESIGN_DIAGNOSTICS.md**.
    *   **`#[gpu]` Code Diagnostics**:
        *   Strict compile-time errors for any usage of non-GPU-Ferra-subset features within a `#[gpu]` scope.
        *   Warnings for potentially inefficient patterns in GPU code (e.g., divergent control flow, uncoalesced memory access, if detectable statically).
        *   Clear errors from the SPIR-V compilation/validation stages if issues are found there.
        *   All such diagnostics should also adhere to **DESIGN_DIAGNOSTICS.md**.
    *   **Performance Hints (Future)**: `lang doctor gpu` could potentially analyze `#[gpu]` code or SPIR-V for common performance anti-patterns.

*   **IDE Integration**:
    *   Syntax highlighting for the `par_for_each` construct and `#[gpu]` attribute.
    *   Type checking and auto-completion for GPU-specific library functions (e.g., from `core::gpu`).
    *   Displaying compiler diagnostics related to vectorization or GPU compilation inline.
    *   Potential for future integrations to trigger GPU kernel analysis or profiling from the IDE.

*   **Documentation and Examples**:
    *   Clear documentation on how to write effective data-parallel `for_each` loops for both CPU SIMD and GPU targets.
    *   Best practices for the "GPU-Ferra" subset.
    *   Detailed guides on host-device data management and kernel invocation.
    *   Examples of common data-parallel algorithms implemented in Ferra.

*   **CI Integration for GPU Builds (Conceptual)**:
    *   CI pipelines should have stages to build and test code paths that include `#[gpu]` attributes.
    *   A basic CI check would ensure that code marked `#[gpu]` successfully compiles to SPIR-V without errors.
    *   More advanced CI could involve running simple test kernels on a CI runner with GPU access (if available) or using a SPIR-V simulator/validator.
    *   **Example CI YAML Excerpt (Conceptual - GitHub Actions)**:
        ```yaml
        # .github/workflows/ci.yml (excerpt)
        # ...
        jobs:
          build_and_test:
            # ...
            steps:
            # ... (other build/test steps) ...
            - name: Build Ferra with GPU Kernels
              run: ferra build --enable-gpu-spirv --target-gpu=vulkan_generic
              # This step fails if `#[gpu]` code doesn't compile to SPIR-V

            - name: Validate SPIR-V (Optional)
              run: |
                # Assuming build artifacts include .spv files in target/spirv/
                # Use spirv-val (from SPIRV-Tools) to validate all generated kernels
                find target/spirv -name '*.spv' -exec spirv-val {} \;
              continue-on-error: false # Fail job if any SPIR-V is invalid
            
            # - name: Run GPU Tests (if CI has GPU runners or simulators)
            #   run: ferra test --include-gpu-tests
        ```

The aim is to make data parallelism a natural extension of Ferra programming, with tools providing helpful feedback, even if direct GPU debugging remains a more advanced topic initially.

## 7. Limitations and Future Work

The initial v0.1 design for data-parallel loops and GPU support in Ferra provides a foundational capability. It will inherently have limitations, and there are many avenues for future expansion and refinement.

*   **Initial Limitations of v0.1**:
    *   **`for_each` Construct**:
        *   May initially only support a limited set of collection types (e.g., `Vector<T>`, slices of primitives or `#[repr(C)]`/`#[repr(SPIRV)]` data types). Support for custom parallel iterators might be basic.
        *   The complexity of the closure body that can be effectively auto-vectorized for CPU SIMD or compiled to an efficient GPU kernel will be limited. Highly divergent control flow or complex data dependencies will likely inhibit performance or be disallowed.
        *   Limited support for reductions, scans, or other complex parallel algorithms beyond element-wise `for_each`.
    *   **"GPU-Ferra" Subset**:
        *   The language subset allowed within `#[gpu]` code (Section 4.2) will be quite restrictive initially, focusing on core arithmetic, simple data structures, and basic control flow.
        *   Dynamic memory allocation, most standard library features (especially I/O, complex collections, string manipulation), and FFI will be unavailable in GPU kernels.
    *   **SPIR-V Generation**:
        *   Initial SPIR-V generation might target a baseline set of capabilities (e.g., Vulkan compute, a common SPIR-V version like 1.3).
        *   Support for advanced SPIR-V extensions or highly specialized GPU features will be limited.
        *   Optimization of generated SPIR-V might initially rely heavily on downstream tools (e.g., `spirv-opt`, vendor drivers) rather than extensive compiler-side SPIR-V optimization passes.
        *   **SPIR-V Generation**:
            *   Initial SPIR-V generation might target a baseline set of capabilities (e.g., Vulkan compute, a common SPIR-V version like 1.3).
            *   Support for advanced SPIR-V extensions or highly specialized GPU features will be limited.
            *   Optimization of generated SPIR-V might initially rely heavily on downstream tools (e.g., `spirv-opt`, vendor drivers) rather than extensive compiler-side SPIR-V optimization passes.
    *   **Host-Device Interaction**:
        *   The initial API for GPU buffer management and kernel launching (`core::gpu`) will be functional but perhaps not as feature-rich or ergonomic as mature GPU compute libraries.
        *   Support for complex memory synchronization patterns or zero-copy memory access (for integrated GPUs) might be limited.
    *   **Debugging**: As noted in Section 6, debugging GPU code will be challenging, relying primarily on external tools or manual techniques.
    *   **Performance Portability**: Achieving optimal performance across a wide range of different GPU architectures with a single SPIR-V target is difficult. Initial v0.1 may not be highly tuned for all possible GPUs.
    *   **Error Handling**: Device-side error reporting from GPU kernels will likely be basic in v0.1.

*   **Potential Future Work and Enhancements**:
    *   **Richer Parallel Iterator Adapters**: Introduce more parallel iterator methods beyond `for_each`, such as `par_map`, `par_filter`, `par_reduce`, `par_scan`, etc., for more expressive data parallelism.
    *   **Expansion of "GPU-Ferra" Subset**: Gradually allow more Ferra language features and standard library components (e.g., more math functions, safe subsets of certain collections) within `#[gpu]` code as feasibility is determined.
    *   **Advanced GPU Features**:
        *   Support for graphics pipelines (vertex, fragment, geometry shaders) if Ferra aims to be used for graphics programming, not just GPGPU compute.
        *   Support for more SPIR-V capabilities and extensions (e.g., subgroup operations, ray tracing if relevant).
        *   Texture and sampler support.
    *   **Improved Host-Device Interaction**:
        *   More sophisticated buffer management (e.g., mapped memory, asynchronous data transfers with explicit synchronization objects).
        *   Stream/queue management for overlapping computation and data transfer.
    *   **Multi-GPU Support**: Explicit APIs for managing and targeting multiple GPU devices.
    *   **Direct Interoperability with Other GPU Compute APIs**: While SPIR-V is the primary target for portability, future consideration for more direct interop layers or bindings for CUDA, Metal, or OpenCL if strong use cases emerge and direct generation isn't sufficient.
    *   **Ferra-Specific GPU Optimizations**: Develop Ferra-aware optimization passes for GPU code, either at the Ferra IR level or during/after SPIR-V generation.
    *   **Improved GPU Debugging**: Investigate ways to improve the GPU debugging experience from within the Ferra ecosystem.
    *   **Automatic Offload Heuristics (Advanced Research)**: Explore compiler heuristics that could automatically decide whether to offload a `par_for_each` loop to the GPU based on data size, computation intensity, and target hardware, rather than requiring an explicit `#[gpu]` attribute for all cases.
    *   **Unified Memory Models**: Better support for platforms with unified host-device memory.

Ferra's journey into data parallelism and GPU support will be iterative, starting with a foundational set of capabilities in v0.1 and expanding based on user needs, hardware evolution, and implementation experience.

## 8. Open Questions / TBD

This section consolidates the "To Be Determined" items identified throughout this document. These represent key design or implementation details that need to be resolved.

*   **(DPGPU-SYNTAX-1)**: **Final `for_each` and `#[gpu]` Syntax**:
    *   The precise syntax for the data-parallel `for_each` construct (e.g., keyword-based vs. method-based `par_iter().for_each()`). If method-based, the exact names and APIs for parallel iterators.
    *   The final syntax for the `#[gpu]` attribute, including any potential parameters (e.g., `#[gpu(target_env="vulkan1.x")]`).
    *   How these will be formally specified in `SYNTAX_GRAMMAR_V0.1.md` and represented in `AST_SPECIFICATION.md`.

*   **(DPGPU-SUBSET-1)**: **Precise "GPU-Ferra" Language Subset**:
    *   A definitive list of allowed and disallowed Ferra language features, data types, and standard library functions within `#[gpu]`-annotated code for the v0.1 SPIR-V target.

*   **(DPGPU-MEMMODEL-1)**: **GPU Memory Model and Host-Device API**:
    *   Detailed specification of the `core::gpu` module APIs for buffer allocation, data transfer (host-to-device, device-to-host), and kernel argument passing.
    *   How Ferra's ownership and borrowing concepts apply to `GpuBuffer<T>` handles and the data they represent.
    *   Synchronization primitives for host-device coordination.
    *   Representation of GPU memory spaces (global, group-shared, private) within GPU-Ferra if explicitly exposed.

*   **(DPGPU-SPIRV-TOOL-1)**: **SPIR-V Generation Toolchain**:
    *   The definitive choice of strategy and tools/libraries for converting Ferra IR (or LLVM IR derived from it) to SPIR-V for v0.1 (e.g., LLVM-SPIR-V backend, `rspirv`, etc.).
    *   Target SPIR-V version and set of required/optional capabilities.

*   **(DPGPU-SIMD-TARGET-1)**: **CPU SIMD Lowering Strategy**:
    *   Specific strategies and IR patterns required to maximize the effectiveness of LLVM's auto-vectorization for `for_each` loops targeting CPUs.
    *   How the existing `vectorize` semantic tag in `IR_SEMANTIC_TAGS.md` interacts with this, or if a new, more specific tag is needed for data-parallel loops.

*   **(DPGPU-ERROR-1)**: **GPU Device-Side Error Handling**:
    *   The primary mechanism for reporting errors or exceptions occurring during kernel execution back to the host Ferra code for v0.1 (e.g., error codes in output buffers, specific return values from launch APIs).

*   **(DPGPU-ATTR-SCOPE-1)**: **Scope of `#[gpu]` Attribute**:
    *   Decision on whether `#[gpu]` can only apply to entire functions, or also to individual `for_each` loops within a CPU function, for v0.1. This has implications for kernel signature generation and code organization.

*   **(DPGPU-IR-REP-1)**: **IR Representation for Parallel Constructs**:
    *   How data-parallel `for_each` loops and `#[gpu]` markers are specifically represented in Ferra IR to facilitate distinct lowering paths for CPU SIMD and GPU SPIR-V.

*   **(DPGPU-RUNTIME-1)**: **GPU Runtime Requirements**:
    *   What minimal runtime support (if any, beyond stdlib `core::gpu` calls) is needed on the host side to manage GPU contexts, kernels, and execution, especially concerning integration with Ferra's main runtime and actor system.

Addressing these TBDs will be crucial for delivering a functional and usable v0.1 implementation of data parallelism and GPU support in Ferra.
---
This document will specify Ferra's approach to data-parallel constructs and GPU code generation via SPIR-V.