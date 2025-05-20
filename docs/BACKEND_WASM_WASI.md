# Ferra Backend: WebAssembly (WASM) & WASI Design v0.1

> **Status:** Initial Draft - Module 2.3 (Steps 2.3.1 - 2.3.4)
> **Last Reviewed:** 2024-07-29
> **Version:** 0.1

## 1. Introduction and Goals

WebAssembly (WASM) is a key compilation target for Ferra, selected for its unique advantages in portability, near-native performance, strong security/sandboxing model, and its ubiquitous support across web browsers and server-side environments (including serverless platforms). Ferra aims to leverage WASM to enable developers to write high-performance, memory-safe code that can run consistently anywhere WASM is supported.

The WebAssembly System Interface (WASI) plays a crucial role by providing a standardized API for WebAssembly modules to interact with the underlying operating system in a portable manner. This allows Ferra programs compiled to WASM to perform system-level operations such as file I/O, network access (in future WASI versions), and accessing environment variables, moving beyond the limitations of browser-only WASM.

This document specifies the design for Ferra's WebAssembly backend and its integration with WASI. The key design goals for this backend are:

1.  **Efficient Code Generation from Ferra IR**: Translate Ferra's Intermediate Representation (IR), as defined in `IR_SPECIFICATION.md`, into compact and performant WASM bytecode. This involves mapping Ferra IR instructions, data types, and control flow to their WASM equivalents.
2.  **Comprehensive WASI Compatibility**: Ensure that Ferra programs can utilize standard system capabilities by mapping relevant Ferra standard library calls (from `STDLIB_CORE_V0.1.md`) to WASI interfaces. This allows Ferra applications to be truly portable across different WASI-compliant runtimes.
3.  **Optimized Binary Size**: Achieve a small footprint for compiled Ferra programs. As per `Steps.md` (Section 11, note on CI linting `max_wasm_size_kb`) and the `comprehensive_plan.md` (Module 2.3.3), a target of **≤ 200 kB baseline** for a minimal Ferra program with core standard library features is a primary objective. This will be pursued through techniques like tree-shaking and potentially lazy standard library segmentation.
4.  **Seamless JavaScript/TypeScript Interoperability**: Provide automatically generated TypeScript definition files (`.d.ts`) for exported Ferra functions and data types, enabling easy and type-safe integration of Ferra WASM modules into JavaScript/TypeScript projects.

This specification will detail the mapping from Ferra IR to WASM, the strategy for WASI integration, approaches for binary size optimization, and the design for TypeScript binding generation. It builds upon the common IR defined in `IR_SPECIFICATION.md` and complements other backend specifications like `BACKEND_LLVM_X86-64.md` by providing an alternative compilation pathway optimized for WASM-specific use cases.

## 2. WASM Code Generation from Ferra IR (Step 2.3.1)

This section details the process of translating Ferra's Intermediate Representation (IR), as defined in `IR_SPECIFICATION.md`, into WebAssembly (WASM) bytecode. The goal is to produce efficient and correct WASM modules that accurately represent the semantics of the original Ferra program.

### 2.1. Mapping Ferra IR to WASM Constructs

*   **Module Structure**: A Ferra compilation unit (crate/module) will be compiled into a single WASM module (`.wasm` file). This module will contain:
    *   **Type Section**: Definitions for any function signatures if not directly representable by WASM valtypes.
    *   **Import Section**: Declarations for imported functions (e.g., from WASI, or from the JavaScript host environment).
    *   **Function Section**: Indices mapping to code bodies for functions defined within the Ferra module.
    *   **Table Section**: Potentially used for function pointers if Ferra supports indirect calls that map to WASM tables (e.g., for dynamic dispatch, though this is an advanced topic for Ferra IR v0.1).
    *   **Memory Section**: Defines the linear memory used by the module.
    *   **Global Section**: For any global variables defined in Ferra IR.
    *   **Export Section**: Public Ferra functions and globals intended to be callable/accessible from the host environment.
    *   **Start Section**: Optionally, a start function to execute upon module instantiation (e.g., for initializing global state or calling a Ferra `main` function if it's not explicitly exported and called by the host).
    *   **Element Section**: For initializing tables (e.g., with function references).
    *   **Data Section**: For initializing regions of linear memory with static data (e.g., string literals, global constants).
    *   **Custom Sections**: Notably, a `ferra.ai` custom section to embed semantic tags as per `IR_SEMANTIC_TAGS.md` (Section 2).
    *   *Conceptual WASM Module Section Order (illustrative EBNF-like representation):*
        ```ebnf
        WasmModule ::= MagicAndVersion
                       (TypeSection)?
                       (ImportSection)?
                       (FunctionSection)?
                       (TableSection)?
                       (MemorySection)?
                       (GlobalSection)?
                       (ExportSection)?
                       (StartSection)?
                       (ElementSection)?
                       (CodeSection)?
                       (DataSection)?
                       (CustomSection "name")* (* e.g., CustomSection "ferra.ai", CustomSection "name" for debug info *)
        ```

*   **Function Representation**: Each Ferra IR function (from `IR_SPECIFICATION.md` §4.2) will be translated into a WASM function. This involves:
    *   Mapping Ferra function parameters and return types to WASM value types (see Section 2.2).
    *   Translating the sequence of basic blocks and IR instructions within each block into a corresponding sequence of WASM instructions.
    *   Local variables in Ferra IR (SSA values, `%result` of instructions) will often map to WASM local variables or live on the WASM operand stack during evaluation.

*   **Basic Blocks and Control Flow**: Ferra IR's basic blocks and control flow instructions map naturally to WASM's structured control flow:
    *   `br label %target` -> `br $label`
    *   `br_cond i1 %condition, label %true_target, label %false_target` -> `(if (then br $true_target) (else br $false_target))` or `br_if $true_target (condition)` followed by `br $false_target` depending on block layout.
    *   Loops in Ferra IR (often constructed with conditional branches and back-edges) will be mapped to WASM `loop` and `block` constructs.
    *   `phi` nodes will be resolved during the translation to ensure correct value flow at points where control flow merges, typically by inserting appropriate `select` instructions or by structuring local variable usage around WASM block parameters/results if applicable.

*   **Instruction Mapping (General Approach)**:
    *   Most Ferra IR arithmetic, logical, and comparison operations (e.g., `add`, `sub`, `eq`, `lt` from `IR_SPECIFICATION.md` §5) have direct or near-direct equivalents in the WASM instruction set (e.g., `i32.add`, `f64.sub`, `i64.eq`, `i32.lt_s`).
    *   Memory operations (`alloca`, `load`, `store`, `getfieldptr`):
        *   `alloca`: For fixed-size stack allocations within a function whose addresses do not escape, `alloca` can often be optimized to WASM local variables. Otherwise, it will involve managing a region of the linear memory stack (e.g., via a stack pointer global).
        *   `load`/`store`: Map to WASM memory load/store instructions (e.g., `i32.load`, `i64.store`) operating on linear memory addresses derived from pointers/offsets.
        *   `getfieldptr`: Maps to address calculation, typically using `i32.add` or `i64.add` on a base pointer with a constant offset for the field.
    *   `call` instructions map to WASM `call` or `call_indirect`.
    *   `cast` instructions map to WASM type conversion instructions (e.g., `i64.extend_i32_s`, `f64.convert_i32_s`, `i32.trunc_f32_s`).

### 2.2. Data Representation

Mapping Ferra's data types to WASM's value types and linear memory model is crucial.

*   **Core Ferra Types to WASM Value Types**:
    *   `Unit ()`: Typically elided for function returns (effectively `void`). If `Unit` must be represented as a value (e.g., as a field in a data structure where layout requires a placeholder, or if a generic type resolves to `Unit` and needs a concrete valtype), it could be represented as an `i32` with value 0, though direct storage is usually optimized away.
    *   `Bool`: `i32` (WASM has no `i1` valtype; 0 for false, 1 for true convention).
    *   `Int` (assuming default 64-bit as per `BACKEND_LLVM_X86-64.md`): `i64`.
        *   Other integer sizes (`Int8`, `Int16`, `Int32`) would map to `i32` (with appropriate sign/zero extension for operations) or `i64` if larger sizes are directly used.
    *   `Float` (assuming default 64-bit): `f64`.
        *   `Float32` would map to `f32`.
    *   `Char` (Unicode scalar value, typically 32-bit): `i32`.

*   **Aggregate Types in Linear Memory**:
    *   **`String`**: Represented as a pointer/offset into linear memory (an `i32` or `i64` depending on memory64 proposal) and a length (`i32`/`i64`). `IR_SPECIFICATION.md` hints at `{i8*, i64}`. For WASM32, this would be `{i32_offset, i32_length}`. The string data itself (UTF-8 bytes) resides in linear memory.
    *   **`data` classes (Structs)**: Laid out contiguously in linear memory. Fields are accessed via base pointer + offset calculations. The layout will follow standard struct packing rules (e.g., alignment of fields).
    *   **Tuples `(T1, T2, ...)`**: Similar to structs, laid out contiguously in memory.
    *   **Arrays/Lists/Vectors `[T]` or `Vector<T>`**: Represented as a structure or a set of values comprising a pointer/offset to the elements in linear memory, a length, and a capacity. For example, `(i32_data_ptr, i32_length, i32_capacity)`.
        *   Elements of type `T` are stored contiguously starting at `data_ptr`.

*   **References (`&T`, `&mut T`)**: Represented as pointers/offsets (`i32` or `i64`) into linear memory, pointing to the borrowed data.

### 2.3. Memory Management in WASM

Ferra, being a non-garbage-collected language with ownership, needs to manage its memory explicitly within WASM's linear memory space.

*   **Linear Memory**: A single linear memory will be used. It can be imported or defined by the module and can grow.
*   **Allocator**: Ferra's memory allocator (responsible for `heap` allocations for `String`, `Vector`, `data` instances not on stack) will be compiled to WASM. Runtime functions like `@ferra_alloc`, `@ferra_free`, `@ferra_realloc` (as mentioned in `BACKEND_LLVM_X86-64.md` and implied for any backend managing memory) will operate on this linear memory.
    *   The specific allocator (e.g., dlmalloc, a custom bump allocator for certain regions) is an implementation detail (see TBD TAG-WASM-1).
*   **Stack**: For function call frames, local variables not fitting registers, and `alloca`'d data, a region of the linear memory will be managed as a stack, typically using a stack pointer global variable, decremented on call and incremented on return.
*   **Ownership and Drop Semantics**: Ferra's `drop` semantics (destructors) must be translated. When an owned value goes out of scope, if it requires deallocation (e.g., a `String` or `Vector`), the compiled Ferra code will invoke the necessary deallocation logic (e.g., calling `@ferra_free` with the correct pointer and size/layout information).

### 2.4. Function Call Conventions

*   **Argument Passing**: Arguments to Ferra functions will be passed via WASM function parameters (which can be `i32`, `i64`, `f32`, `f64`). For aggregate types passed by value that are too large for registers, they might be passed indirectly via a pointer to a copy in linear memory, or their fields might be destructured if feasible.
*   **Return Values**: Return values also use WASM function results. Multiple return values (e.g., from a Ferra function returning a tuple) can be handled by WASM's multi-value proposal, or by returning a pointer to a structure in memory.
*   **Exported Ferra Functions**: Functions marked as public/exported in Ferra will be listed in the WASM module's export section, making them callable from the host (e.g., JavaScript).
*   **Imported Functions (Host/WASI)**: Ferra code calling external functions (e.g., WASI functions, JavaScript functions provided by the host) will use WASM `call` instructions targeting imported functions defined in the import section.
    *   The signatures of these imported functions must match what the WASM module expects and what the host provides.

### 2.5. Ferra Runtime Components for WASM (Minimal)

While Ferra aims for minimal runtime, some components will likely be compiled into the WASM module:

*   **Memory Allocator**: As discussed (e.g., `@ferra_alloc`, `@ferra_free`).
*   **Panic Handling**: A mechanism for handling Ferra panics. This might involve: (see TBD TAG-WASM-2)
    *   A `@ferra_panic(message_ptr, message_len, file_ptr, file_len, line, col)` function compiled to WASM.
    *   This function might print to stderr (via WASI `fd_write`) and then trap (`unreachable` in WASM) or call a specific host import to signal the panic.
*   **Type Information/RTTI (Minimal)**: If features like dynamic type checks or reflection are supported (even minimally), some runtime type information might be needed, though this is typically avoided for WASM to keep binaries small.
*   **String/Collection Helpers**: Potentially some low-level helper functions for common `String` or `Vector` operations if they are complex to inline efficiently in WASM directly from IR for all cases (e.g., certain Unicode processing for strings, complex reallocation logic for vectors). These would be internal to the Ferra-generated WASM module and not necessarily part_of the public stdlib API directly callable by users from Ferra.

This initial pass at IR-to-WASM generation will focus on correctness and laying the groundwork for optimizations and advanced features.

## 3. WASI Compatibility (Step 2.3.2)

The WebAssembly System Interface (WASI) provides a crucial bridge for Ferra programs compiled to WASM to interact with the underlying operating system in a standardized way. This section outlines Ferra's strategy for WASI compatibility, enabling capabilities like console I/O, file system access, and more.

### 3.1. Identifying Core WASI Modules and Interfaces

Ferra will initially target a stable version of WASI, likely starting with **`wasi_snapshot_preview1`** (or its closest successor if a new stable version emerges during implementation). Support for later WASI proposals and features (e.g., wasi-threads, wasi-sockets) can be added incrementally.

Key WASI interfaces that Ferra's standard library (especially `STDLIB_CORE_V0.1.md` and its expansions) will need to leverage include:

*   **Console I/O**: Essential for `println`, `print`, `eprintln`, and `read_line`.
    *   `fd_write(fd: Fd, iovs: CiovecArray) -> Result<Size>`: For writing to standard output (typically `fd=1`) and standard error (typically `fd=2`).
    *   `fd_read(fd: Fd, iovs: IovecArray) -> Result<Size>`: For reading from standard input (typically `fd=0`).
*   **File System Access**: For future file I/O operations in the standard library.
    *   `path_open(dirfd: Fd, dirflags: Lookupflags, path: String, oflags: Oflags, fs_rights_base: Rights, fs_rights_inheriting: Rights, fdflags: Fdflags) -> Result<Fd>`: To open files and directories.
    *   `fd_read(fd: Fd, iovs: IovecArray) -> Result<Size>`: To read from files.
    *   `fd_write(fd: Fd, iovs: CiovecArray) -> Result<Size>`: To write to files.
    *   `fd_close(fd: Fd) -> Result<()>`: To close file descriptors.
    *   `fd_seek(fd: Fd, offset: Filedelta, whence: Whence) -> Result<Filesize>`: To seek within files.
    *   `path_filestat_get(dirfd: Fd, flags: Lookupflags, path: String) -> Result<Filestat>`: To get file metadata.
    *   `path_create_directory(dirfd: Fd, path: String) -> Result<()>`
    *   `path_remove_directory(dirfd: Fd, path: String) -> Result<()>`
    *   `path_unlink_file(dirfd: Fd, path: String) -> Result<()>`
    *   `fd_readdir(fd: Fd, buf: Buffer, cookie: Dircookie) -> Result<Size>`: To read directory entries.
*   **Clocks**: For time-related functionalities.
    *   `clock_time_get(id: Clockid, precision: Timestamp) -> Result<Timestamp>`: To get current time (e.g., monotonic, wall clock).
*   **Randomness**: For generating random numbers.
    *   `random_get(buf: Buffer) -> Result<()>`: To fill a buffer with cryptographically secure random bytes.
*   **Environment Variables**: For accessing the host environment.
    *   `environ_sizes_get() -> Result<(Size, Size)>`
    *   `environ_get(environ: Pointer<Pointer<u8>>, environ_buf: Pointer<u8>) -> Result<()>`
*   **Command-Line Arguments**: For accessing program arguments.
    *   `args_sizes_get() -> Result<(Size, Size)>`
    *   `args_get(argv: Pointer<Pointer<u8>>, argv_buf: Pointer<u8>) -> Result<()>`
*   **Process Control**: For exiting the program.
    *   `proc_exit(rval: Exitcode) -> !`: To terminate the current process.

These interfaces will be imported by the Ferra WASM module from the `wasi_snapshot_preview1` (or equivalent) module provided by the WASI runtime.

### 3.2. Mapping Ferra STDLIB to WASI Calls

The Ferra standard library will provide an abstraction layer over these WASI calls, ensuring that Ferra code remains portable across different environments (WASI and potentially non-WASI native targets).

*   **Console I/O (`STDLIB_CORE_V0.1.md`)**:
    *   `println(message: String)`: Will format the string and use `fd_write` to WASI `stdout` (`fd=1`).
    *   `eprintln(message: String)`: Will use `fd_write` to WASI `stderr` (`fd=2`).
    *   `read_line() -> Result<String, IOError>`: Will use `fd_read` from WASI `stdin` (`fd=0`), handling buffering and line termination.
    *   The `IOError` type from `STDLIB_CORE_V0.1.md` will be used to represent errors returned by these WASI calls, mapping WASI `errno` values to appropriate `IOError.code` and messages.
*   **Future File I/O**: Ferra stdlib functions for file operations (e.g., `File::open`, `file.read`, `file.write`) will directly map to the corresponding `path_open`, `fd_read`, `fd_write`, `fd_close`, etc., WASI calls.
*   **Other Stdlib Features**: As Ferra's standard library grows to include time, random number generation, environment access, etc., these functionalities will be implemented by calling the respective WASI interfaces listed in 3.1.

An internal `sys` or `sys_wasi` module within the Ferra standard library will likely encapsulate the direct FFI-like calls to the imported WASI functions, providing a safer, idiomatic Ferra API to the rest of the standard library and user code.

### 3.3. WASI Capabilities and Security

WASI is designed with a capability-based security model. A WASM module running in a WASI environment is typically granted specific permissions (e.g., access to certain directories or environment variables) by the host runtime, rather than having unrestricted access.

*   **Ferra's Approach**: Ferra programs compiled to WASM/WASI will operate within the permissions granted by the host WASI runtime.
    *   The Ferra compiler itself will not embed or enforce these capabilities directly into the WASM binary beyond what WASI requires for imports.
    *   It will be the responsibility of the WASI runtime (e.g., Wasmtime, Wasmer, Node.js WASI support) and the user executing the Ferra WASM module to configure these permissions. The Ferra package's `manifest.perms` (defined in **SECURITY_MODEL.md**, Section 3) will serve as the primary declaration of intended capabilities, which Ferra tooling (e.g., a Wasm runner or deployment scripts) can then use to automatically configure the WASI runtime environment. This provides a bridge from Ferra's high-level permission declarations to WASI's lower-level capability granting.
*   **Standard Library Behavior**: The Ferra standard library functions that rely on WASI calls (e.g., file access) will return appropriate `IOError`s if an operation is attempted for which the module lacks the necessary capability (as signaled by the WASI runtime).
*   **Alignment with Future Ferra Security Model**: Ferra's planned capability-based permissions (`manifest.perms` from `SECURITY_MODEL.md` - Module 3.4) will define what a Ferra *package* declares it needs. When compiling to WASM/WASI:
    *   The manifest could serve as a hint to users or tools about what WASI capabilities to grant the module.
    *   There is no direct enforcement mechanism from Ferra's manifest to WASI runtime capabilities at the WASM binary level itself; they are separate layers. However, tools could bridge this, e.g., a `ferra run my_module.wasm` command could read the manifest and configure the WASI runtime accordingly.
*   **Error Reporting**: If a Ferra stdlib call fails due to a WASI permission error (e.g., `EACCES`, `ENOPERM`), this will be translated into a specific `IOError` indicating a permission issue.

This approach ensures that Ferra programs are good citizens in the WASI ecosystem, respecting its security model while providing the necessary system interactions for a wide range of applications.

## 4. Binary Size Optimization (Target ≤ 200 kB Baseline) (Step 2.3.3)

Achieving a small binary footprint is a critical goal for Ferra's WebAssembly backend, especially for web delivery and resource-constrained environments. The `comprehensive_plan.md` (Module 2.3.3) and `Steps.md` (Section 11, note on `max_wasm_size_kb`) set a target of **≤ 200 kB baseline** for a minimal Ferra program including core standard library features. This target will be enforced by CI checks (e.g., a `max_wasm_size_kb` lint rule as mentioned in `Steps.md`) to ensure ongoing compliance. This section outlines strategies to meet this target.

### 4.1. Profile-Guided Optimization (PGO) for WASM

While PGO is a powerful technique, its application to WASM for initial versions of Ferra might be an advanced topic. However, the design should not preclude its future use.

*   **Conceptual Approach**: PGO involves compiling the program with instrumentation, running it with typical workloads to gather execution profiles (e.g., hot paths, frequently called functions), and then recompiling using this profile data to make more informed optimization decisions (e.g., better inlining, code layout).
*   **Profile Collection**: For Ferra WASM modules, profile collection could potentially be done using WASM runtimes that support PGO data generation or through custom instrumentation if necessary.
*   **Application**: PGO could help in optimizing stdlib functions that are frequently used or critical user code paths for size and speed.
*   **Initial Focus**: For v0.1, the primary focus will be on direct size reduction techniques. PGO is a future enhancement consideration for further refinement.

### 4.2. Tree-Shaking and Dead Code Elimination (DCE)

This is a fundamental strategy for minimizing binary size.

*   **Granularity**: The Ferra compiler and linker must work together to eliminate any code (from both user libraries and the Ferra standard library) that is not reachable from the program's entry points (e.g., the exported `main` function or other exported functions).
*   **Compiler Passes**: The Ferra compiler (during IR-to-WASM generation) should perform basic DCE for unreachable basic blocks and functions within a module before linking.
*   **Linker-Level Tree-Shaking**: Utilizing a WASM-aware linker that supports aggressive tree-shaking is essential.
    *   **LLD (LLVM Linker)**: If the Ferra-to-WASM pipeline involves an LLVM step (even if just for linking objects compiled from Ferra IR by a custom WASM backend, or if Ferra IR is first translated to LLVM IR then to WASM), LLD's WASM support can perform effective DCE.
    *   **Custom Linker/Tooling**: If a more direct Ferra IR -> WASM text/binary toolchain is used, it must incorporate or be followed by a tool that performs robust DCE based on the WASM module's import/export graph and call graph.
*   **Standard Library Design**: The Ferra standard library should be designed with tree-shaking in mind, favoring smaller, more modular functions where possible to allow unused parts to be easily eliminated.

### 4.3. Lazy Standard Library Segmentation

This is a more advanced technique to reduce the *initial* download/load size, potentially by splitting the standard library or parts of the application into multiple WASM modules or segments that can be loaded on demand. This is distinct from tree-shaking which removes unused code entirely.

*   **Concept**: Identify parts of the standard library that are less commonly used or only needed for specific features. These could be compiled into separate, smaller WASM modules.
*   **Dynamic Linking/Loading**: The main Ferra WASM module would then dynamically link against or load these auxiliary stdlib modules if and when their functionality is required. WASM has proposals for dynamic linking and module linking that could support this.
*   **Challenges**: This adds complexity to the build system, runtime, and deployment. Managing dependencies and interfaces between dynamically linked WASM modules requires careful design.
*   **Initial v0.1 Strategy**: For the initial ≤ 200 kB target, the focus will be primarily on aggressive tree-shaking of a monolithically linked standard library. Lazy segmentation is a strong candidate for future optimization if the baseline size with tree-shaking alone proves insufficient or for applications requiring even smaller initial footprints.

### 4.4. Other Optimization Techniques

*   **`wasm-opt` (Binaryen)**: The Binaryen toolkit includes `wasm-opt`, a powerful post-compilation optimizer for WASM modules. It can perform a wide range of size and speed optimizations, including DCE, inlining, instruction simplification, and more. Integrating `wasm-opt` as a final step in the Ferra-to-WASM build pipeline is highly recommended.
*   **Compiler Optimizations**: The Ferra compiler, during IR-to-WASM translation, should implement standard compiler optimizations that also benefit size, such as:
    *   Constant folding and propagation.
    *   Instruction simplification.
    *   Inlining of small functions (judiciously, as over-inlining can increase size).
*   **Data Segment Optimization**: Minimizing the size of the data section by, for example, efficiently representing string literals or other static data.
*   **Code Generation Choices**: Selecting WASM instructions and patterns that are more compact where semantically equivalent alternatives exist.
*   **Avoidance of Unnecessary Runtime**: Keeping the Ferra runtime components compiled into WASM (Section 2.5) as minimal as strictly necessary.

Meeting the ≤ 200 kB baseline will likely require a combination of these strategies, with a strong emphasis on effective tree-shaking and the use of tools like `wasm-opt` in the initial versions.

## 5. Auto-generation of TypeScript Bindings (Step 2.3.4)

To facilitate seamless integration of Ferra WASM modules into JavaScript and TypeScript ecosystems, the Ferra compiler will support the automatic generation of TypeScript definition files (`.d.ts`). This provides type safety and developer convenience when interacting with Ferra code from JS/TS.

### 5.1. Binding Generation Trigger and Scope

*   **Trigger**: TypeScript binding generation will likely be triggered by a compiler flag (e.g., `--emit-ts-bindings`, `--ts-output-dir <dir>`).
*   **Scope of Exposed Constructs**: Only `public` Ferra functions explicitly marked for export in the WASM module will have corresponding TypeScript bindings generated.
    *   Data types (`data` classes, tuples) used as parameters or return types of these exported public functions will also have their TypeScript type definitions generated.
    *   Internal Ferra types or non-exported functions will not be included in the generated bindings.
*   **Module Granularity**: Bindings will be generated per Ferra module (compilation unit) that is compiled to WASM.

### 5.2. Mapping Ferra Types to TypeScript Types

The following mapping will be used to translate Ferra types to their TypeScript equivalents:

| Ferra Type                      | TypeScript Type                       | Notes                                                                                                |
|---------------------------------|---------------------------------------|------------------------------------------------------------------------------------------------------|
| `Unit`                          | `void`                                | For function return types.                                                                           |
| `Bool`                          | `boolean`                             |                                                                                                      |
| `Int` (e.g., 64-bit default)    | `number` or `bigint`                  | `number` if within JS safe integer range; `bigint` for full 64-bit range if enabled/supported.         |
| `Int8`, `Int16`, `Int32`        | `number`                              |                                                                                                      |
| `Float` (e.g., 64-bit default)  | `number`                              | Corresponds to `f64`.                                                                                |
| `Float32`                       | `number`                              | Corresponds to `f32`.                                                                                |
| `Char` (Unicode Scalar Value)   | `string`                              | A single-character string, as JS/TS lacks a distinct char type. Alternatively, `number` (codepoint). |
| `String`                        | `string`                              | Marshalling required (see 5.3).                                                                      |
| `data MyData { ... }`           | `interface MyData { ... }` or `class` | Fields mapped according to their types.                                                              |
| `Vector<T>` or `[T]` (List)     | `Array<TS_T>` or `TS_T[]`             | Where `TS_T` is the TypeScript mapping of Ferra type `T`. Marshalling required.                      |
| `(T1, T2)` (Tuple)              | `[TS_T1, TS_T2]` (Tuple type)         | Marshalling may be required if passed as a structured object.                                        |
| `Option<T>`                     | `TS_T | null` or `TS_T | undefined`   | Depending on convention.                                                                             |
| `Result<T, E>`                  | `interface Ok<T> { kind: "ok"; value: T; }`<br/>`interface Err<E> { kind: "err"; error: E; }`<br/>`type Result<T,E> = Ok<T> | Err<E>;` | Discriminated union for type safety. |
| `fn(A, B) -> R` (as parameter)  | `(a: TS_A, b: TS_B) => TS_R`          | For callbacks passed from JS to Ferra.                                                               |

*   **Note on `Int` (64-bit)**: JavaScript's `number` type cannot safely represent all 64-bit integers. If Ferra's `Int` is 64-bit, functions accepting or returning these may need to use `bigint` in TypeScript, requiring explicit handling by the JS/TS caller. The binding generator might offer a flag to control this behavior or default to `number` with clear documentation about potential precision loss for very large integers.
*   **Note on `Char`**: While `string` is common, mapping to `number` (Unicode code point) could also be an option if direct numerical manipulation of character codes is expected.

### 5.3. Exposing Exported Ferra Functions & Data Marshalling

*   **Function Signatures**: Exported Ferra functions will be represented as functions in the TypeScript definition file with appropriately mapped parameter and return types.
    ```typescript
    // Example: Ferra: pub fn greet(name: String) -> String
    export function greet(name: string): string;
    ```
*   **Data Marshalling**: Interacting with WASM linear memory from JavaScript/TypeScript requires careful data marshalling, especially for complex types like strings, arrays, and data classes.
    *   **Strings**: Ferra functions taking or returning `String` will typically involve:
        *   **From JS to WASM**: The JS string needs to be encoded (e.g., UTF-8), written into the WASM linear memory, and a pointer/length pair passed to the Ferra WASM function.
        *   **From WASM to JS**: The Ferra WASM function returns a pointer/length pair for a string in its linear memory. JS code reads the UTF-8 bytes from that memory region and decodes them into a JS string.
    *   **Arrays/Vectors**: Similar to strings, involves allocating memory in WASM, copying data, and passing pointers/lengths, or vice-versa.
    *   **Data Classes/Structs**: For complex data structures passed by value, they would typically be serialized into linear memory (e.g., as a sequence of fields or a flat byte buffer) by one side and deserialized by the other. Alternatively, if passed by reference, only a pointer is exchanged.
*   **Helper Functions**: The generated TypeScript bindings may include or recommend the use of small JavaScript helper functions to encapsulate common marshalling logic, making it easier for developers to call Ferra WASM functions.
    *   For example, a `readFerraString(pointer: number, length: number): string` helper.
    *   Tools like `wasm-bindgen` (from the Rust ecosystem) provide extensive examples of such helper generation, which Ferra can draw inspiration from.

### 5.4. Output Format and Tooling

*   **`.d.ts` File Generation**: The primary output will be a TypeScript definition file (`.d.ts`) for each compiled Ferra module.
*   **Module Loading**: The bindings will assume a standard way of loading and instantiating the WASM module (e.g., using `WebAssembly.instantiateStreaming` or a similar API).
    *   The generated `.d.ts` might include a top-level function or class that handles the WASM module instantiation and exposes the typed Ferra functions.
    ```typescript
    // Conceptual structure of a generated .d.ts file
    export interface MyFerraData { field: number; /* ... */ }

    export function exported_ferra_function(arg1: string, arg2: number): MyFerraData;
    // ... other exported functions ...

    // Optional: Helper class/functions for loading and interacting with the WASM module
    // export class FerraModule {
    //     constructor(module: WebAssembly.Module, instance: WebAssembly.Instance);
    //     public greet(name: string): string;
    //     // ... other methods wrapping exported Ferra functions ...
    // }
    // export function loadFerraModule(wasmPath: string): Promise<FerraModule>;
    ```
*   **Documentation**: Comments from Ferra source code (e.g., doc comments on public functions/types) could potentially be carried over into the generated `.d.ts` files as JSDoc comments to improve usability.

This auto-generation capability aims to significantly lower the barrier to using Ferra code within the wider JavaScript/TypeScript ecosystem, promoting interoperability and type safety.

## 6. Tooling and Build Workflow Considerations

Effective tooling and a smooth build workflow are essential for developers targeting WebAssembly with Ferra.

*   **Compiler Flags**: The Ferra compiler (`ferrac`) will need specific flags to manage WASM/WASI compilation:
    *   `--target wasm32-wasi`: Specifies the WASM32 target with WASI environment (or `wasm64-wasi` if/when 64-bit memory is a target).
    *   `--emit-ts-bindings`: Triggers the generation of TypeScript definition files (as detailed in Section 5).
    *   `--ts-output-dir <directory>`: Specifies the output directory for generated TypeScript bindings.
    *   `--wasm-opt-level <0-4|s|z>`: (Conceptual) Controls the level of optimization specifically for the WASM output, potentially mapping to `wasm-opt` tool levels (e.g., 0=none, 1=basic, 2=default, 3=aggressive, s=size, z=extra_size_aggressive).
    *   Flags to control WASM features used (e.g., multi-value, bulk-memory, reference types) if they are not universally enabled by default.
*   **Build System Integration**: Ferra's build system (if one is developed beyond simple compiler invocations, or as part of the `PACKAGE_MANAGER_SPEC.md` - Module 2.4) should seamlessly support WASM/WASI targets:
    *   Easy configuration of target triples.
    *   Management of WASM-specific build profiles (e.g., a `release-wasm` profile that enables size optimizations and `wasm-opt`).
    *   Handling of TypeScript binding generation as part of the build process.
*   **Debugging WASM Modules**: Providing a good debugging experience for Ferra code running as WASM is important.
    *   **Source Maps**: The compiler should aim to generate DWARF debug information that can be processed (e.g., by tools like `wasm-pack` or future WASM debugging standards) into source maps usable by browser developer tools or specialized WASM debuggers. This allows stepping through original Ferra source code.
    *   **Integration with Debuggers**: Compatibility with debuggers that support the WASM DWARF extension (e.g., in Chrome DevTools, LLDB with WASM support).
*   **WASI Runtime Execution**: For running Ferra WASI applications locally, developers will use standalone WASI runtimes like Wasmtime, Wasmer, or Node.js. Documentation should guide users on this.
*   **Testing**: The Ferra testing framework should support running tests for code compiled to WASM/WASI, potentially by executing test binaries within a WASI runtime.

## 7. Open Questions / TBD

*   (TAG-WASM-1) Specific allocator choice for Ferra's heap in WASM.
*   (TAG-WASM-2) Detailed strategy for handling panics and stack traces in WASM.
*   (TAG-WASM-3) Integration with WASM garbage collection proposal (if/when it becomes relevant and Ferra wishes to use it for managed host references).
*   (TAG-WASM-4) Advanced JS/TS interop features (e.g., direct DOM manipulation, async integration with JS Promises beyond basic function calls).
*   (TAG-WASM-5) Specific versioning of WASI to target.

## 8. Future Enhancements and Considerations

Beyond the initial v0.1 specification, several areas for future enhancements and ongoing consideration for the WASM/WASI backend include:

*   **Support for Evolving WASI Proposals**: As new WASI proposals (e.g., wasi-threads, wasi-sockets, wasi-http) mature and become standardized, Ferra's WASM backend and standard library should be updated to incorporate and leverage them.
*   **Advanced WebAssembly Features**: The backend should be prepared to adopt and leverage advanced or upcoming WASM features as they gain broad support and prove beneficial for Ferra (e.g., SIMD, reference types, tail calls, garbage collection integration for host types, component model).
*   **Security and Permissions**: Continuously refine how Ferra's security model interacts with WASI capabilities and evolving WASM security features to ensure Ferra programs can operate securely and with fine-grained permissions in diverse environments.
*   **TypeScript Binding Generation**: Enhance TypeScript binding generation with more sophisticated marshalling, support for more complex Ferra types, and potentially more configurable helper code generation.
*   **Binary Size and Performance Optimization**: Ongoing efforts to refine compilation strategies, improve tree-shaking, explore PGO more deeply, and adopt new WASM-specific optimizations to further enhance performance and reduce binary sizes.
*   **Debugging Experience**: Continuously improve the debugging experience for Ferra-on-WASM, including better source map generation and deeper integration with WASM debugging tools and standards.

---
This document outlines the design and implementation considerations for Ferra's WebAssembly backend and its integration with WASI. It provides a comprehensive overview of the mapping from Ferra IR to WASM, the strategy for WASI compatibility, approaches for binary size optimization, and the design for TypeScript binding generation. It also covers tooling and build workflow considerations, open questions, and areas for future enhancement.