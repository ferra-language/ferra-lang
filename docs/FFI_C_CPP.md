# Ferra FFI (Initial C/C++) Design v0.1

> **Status:** Initial Draft - Module 2.6 (Step 2.6.1)

## 1. Introduction and Goals

The Foreign Function Interface (FFI) is a critical component of any modern systems programming language, enabling it to leverage the vast ecosystem of existing code and to be integrated into larger projects written in other languages. For Ferra, providing robust FFI capabilities, initially focused on C and C++, is essential for practical adoption and utility.

*   **Purpose of FFI in Ferra**:
    *   To allow Ferra programs to call functions written in C and C++, and to be called by C/C++ code.
    *   To enable the use of existing high-performance C/C++ libraries (e.g., for OS interaction, hardware access, specialized computations, GUI toolkits).
    *   To facilitate gradual adoption of Ferra by allowing new Ferra components to be integrated into existing C/C++ codebases.
    *   To support the creation of Ferra bindings for C/C++ libraries.

*   **Goals for C/C++ Interoperability**:
    *   **Seamlessness**: Make calling C functions from Ferra, and exposing Ferra functions to C, feel as natural as possible within the constraints of safety and ABI differences.
    *   **Safety**: Provide mechanisms to manage the inherent unsafety of FFI boundaries, clearly demarcating `unsafe` regions and guiding developers towards safe wrapper patterns. Ferra's ownership and borrowing system should help in reasoning about resource management across FFI calls.
    *   **Performance**: Ensure that FFI calls have minimal overhead, approaching the performance of direct C calls where possible.
    *   **Ergonomics**: Offer clear syntax and tooling for FFI declarations and data marshalling.
    *   **Portability**: Aim for FFI mechanisms that are portable across common platforms where C/C++ and Ferra are supported.

*   **High-Level Principles**:
    *   **C ABI as Lingua Franca**: Prioritize compatibility with the C Application Binary Interface (ABI) as the most common denominator for interoperability with C and C++.
    *   **Explicit `unsafe`**: Interactions across the FFI boundary are inherently `unsafe` as the Ferra compiler cannot verify the behavior or memory safety of external C/C++ code. All FFI calls from Ferra will require an `unsafe` context.
    *   **Clear Ownership and Lifetime Semantics**: Define clear rules for data ownership and lifetimes when passing data between Ferra and C/C++, especially for memory allocated on either side.

*   **Scope for v0.1 (Initial C/C++ Focus)**:
    *   The initial FFI design (v0.1) will focus exclusively on interoperability with C.
    *   Interoperability with C++ will primarily be achieved by C++ code exposing a C-compatible API (`extern "C"` functions and data structures). Direct interop with C++ features (classes, templates, exceptions) is out of scope for v0.1.
    *   Support for other languages (Rust, Python, JVM, .NET, etc.) is a longer-term goal (Phase 4 of `comprehensive_plan.md`).

## 2. Core Concepts

Interacting with external C/C++ code from Ferra, or vice-versa, requires understanding a few fundamental concepts that bridge the two worlds. These concepts revolve around how functions are called, how data is represented, and how safety is managed.

*   **ABI (Application Binary Interface) Considerations**:
    *   **Targeting C ABI**: Ferra's FFI will primarily target the C ABI of the platform it's compiling for. The C ABI is a widely adopted standard that defines low-level details such as:
        *   **Calling Conventions**: How function arguments are passed (e.g., in registers, on the stack) and how return values are handled.
        *   **Data Type Representation**: How basic data types (integers, floats, pointers) are laid out in memory.
        *   **Name Mangling**: How symbol names (functions, globals) are represented for the linker (C typically has minimal to no name mangling for `extern "C"` functions).
    *   **Platform Dependence**: ABIs are platform-specific (e.g., System V AMD64 ABI for Linux/macOS, Microsoft x64 ABI for Windows). Ferra's FFI implementation must correctly adhere to the target platform's C ABI.
    *   **C++ ABI**: Direct compatibility with C++ ABIs (which are more complex due to name mangling for classes, templates, overloading, etc.) is **out of scope for v0.1**. C++ code wishing to interoperate with Ferra must expose a C-compatible interface (usually via `extern "C"` blocks in C++).

*   **The Role of `unsafe` in FFI**:
    *   **Inherent Unsafety**: The Ferra compiler cannot verify the correctness or memory safety of external C/C++ code. C/C++ code can have bugs, violate memory safety (e.g., buffer overflows, use-after-free), or uphold invariants that Ferra is unaware of.
    *   **`unsafe` Keyword**: Consequently, any Ferra code that directly calls an external C/C++ function, or any Ferra function exposed to be called by C/C++, must be marked with the `unsafe` keyword or enclosed within an `unsafe` block. This serves several purposes:
        1.  It signals to the Ferra developer that they are responsible for upholding any safety invariants required by the FFI call.
        2.  It allows the Ferra compiler to permit operations that would normally be disallowed for safety reasons (e.g., dereferencing raw pointers).
    *   **Minimizing `unsafe`**: While `unsafe` is necessary at the FFI boundary, a common Ferra practice will be to write safe abstractions (wrapper functions/modules) around `unsafe` FFI declarations. These wrappers can encapsulate the unsafety and present a safe, idiomatic Ferra API to the rest of the Ferra codebase.

*   **Overview of Type Mapping and Data Marshalling Challenges**:
    *   **Type Systems Differences**: Ferra and C/C++ have different type systems. For example, Ferra has built-in `String` and `Vector<T>` types with ownership and lifetime semantics, while C uses null-terminated `char*` for strings and manual memory management for arrays.
    *   **Data Representation**: Data types must be mapped between Ferra and their C-compatible equivalents. This includes ensuring compatible memory layouts for structs/`data` classes (`#[repr(C)]` will be crucial).
    *   **Data Marshalling**: The process of converting data from one language's representation to another's at the FFI boundary. This can involve:
        *   Converting Ferra `String` to `char*` (and managing memory, null termination).
        *   Passing Ferra `Vector` data as a pointer and length to C.
        *   Handling differences in how enums or booleans might be represented.
    *   **Ownership and Lifetimes**: A key challenge is managing data ownership and lifetimes across the FFI boundary.
        *   Who owns memory allocated on either side?
        *   When is it safe to pass a pointer from Ferra to C, or from C to Ferra?
        *   How are Ferra's borrow checking rules respected or carefully bypassed at the FFI layer?
    *   These challenges will be addressed in detail in subsequent sections on type mapping and memory management.

## 3. Calling C from Ferra

### 3.1. Declaring External C Functions

To call C functions from Ferra, they must first be declared to the Ferra compiler. This involves specifying their signature, including parameter types and return type, and indicating that they are external C functions.

*   **Syntax for Declaration**:
    *   Ferra will use a syntax similar to Rust's for declaring external C functions, likely an `extern "C"` block or an attribute. The `extern "C"` block is a common convention.
    *   **Conceptual Syntax**:
        ```ferra
        // Option 1: Extern block (Rust-like)
        extern "C" {
            fn c_function_name(arg1: CType1, arg2: CType2) -> CReturn;
            fn another_c_function(input_ptr: *const char);
            // ... other C function declarations
        }

        // Option 2: Attribute per function (Alternative, less common for grouping)
        // #[ffi(linkage="C")]
        // fn c_function_name(arg1: CType1, arg2: CType2) -> CReturn;
        ```
    *   The `extern "C"` block approach is generally preferred for grouping related C function declarations.
    *   The TBD (FFI-C-1) "Exact syntax for `extern "C"` blocks/attributes in Ferra" will need to be resolved by referencing `SYNTAX_GRAMMAR_V0.1.md` or proposing an extension if current attribute syntax isn't sufficient. For now, we assume a syntax like `extern "C" { ... }`. The precise grammar for `extern` blocks and the allowed linkage strings like "C" should be formally defined in `SYNTAX_GRAMMAR_V0.1.md` (related to FFI-C-1).

*   **Specifying Function Signatures**:
    *   Inside the `extern "C"` block, Ferra function signatures are used, but the types specified must be Ferra types that are FFI-safe and correspond to the C types.
    *   For example, a C `int` might map to Ferra `Int` (or a more specific `i32`/`c_int` type alias if available), `char*` to `*const char` (a raw pointer type in Ferra).
    *   Variadic C functions (e.g., `printf(const char* format, ...)`):
        *   Declaring and calling variadic C functions safely from Ferra is complex.
        *   For v0.1, direct calling of variadic C functions might be:
            *   Unsupported: Requiring C wrappers for such functions.
            *   Supported with strong caveats and `unsafe` requirements, using a special syntax (e.g., `...` in the Ferra declaration). This is a common FFI challenge.
        *   **Initial Stance for v0.1**: Defer full variadic support. Users should create simple C wrappers if needed.

*   **Linkage Names (Symbol Names)**:
    *   By default, the Ferra function name used in the `extern "C"` block (e.g., `c_function_name`) is assumed to be the symbol name that the linker will look for in the C library. C typically does not mangle names in `extern "C"` contexts.
    *   **`#[link_name = "..."]` Attribute**: If the C symbol name differs from the desired Ferra function name, an attribute like `#[link_name = "actual_c_symbol_name"]` can be used on the Ferra declaration.
        ```ferra
        extern "C" {
            #[link_name = "my_c_library_print_version"]
            fn print_lib_version();
        }
        ```
    *   This allows Ferra code to use idiomatic names while still linking correctly to C libraries with potentially different naming conventions.

*   **Linking with C Libraries**:
    *   The Ferra build system will need to be instructed to link against the C static or dynamic library that provides these functions. This is covered in Section 7 "Build System and Linker Integration."
    *   For common system libraries, the linker might find them automatically. For other libraries, specific linking instructions will be needed.

All functions declared in an `extern "C"` block are implicitly `unsafe` to call from Ferra, as the Ferra compiler cannot verify their internal implementation.

### 3.2. Data Type Mapping (Ferra <- C)

When declaring an external C function in Ferra, the C types in its signature must be mapped to corresponding Ferra types that are FFI-safe and accurately represent the C data. Ferra will provide a set of built-in types and mechanisms for this mapping.

*   **Quick Reference Table: C Type ⇄ Ferra FFI Alias ⇄ Base Ferra Type**

    | C Type                 | Recommended Ferra FFI Alias | Base Ferra Type(s) | Notes                                      |
    |------------------------|-----------------------------|--------------------|--------------------------------------------|
    | `char`                 | `c_char`                    | `i8` / `u8`        | Platform dependent signedness              |
    | `signed char`          | `c_schar`                   | `i8`               |                                            |
    | `unsigned char`        | `c_uchar`                   | `u8`               |                                            |
    | `short`                | `c_short`                   | `i16`              |                                            |
    | `unsigned short`       | `c_ushort`                  | `u16`              |                                            |
    | `int`                  | `c_int`                     | `i32`              | Common, but verify target ABI              |
    | `unsigned int`         | `c_uint`                    | `u32`              | Common, but verify target ABI              |
    | `long`                 | `c_long`                    | `i32` / `i64`      | Platform dependent (LP32/ILP32 vs LP64)    |
    | `unsigned long`        | `c_ulong`                   | `u32` / `u64`      | Platform dependent                         |
    | `long long`            | `c_longlong`                | `i64`              | Typically 64-bit                           |
    | `unsigned long long`   | `c_ulonglong`               | `u64`              | Typically 64-bit                           |
    | `float`                | `c_float`                   | `f32`              |                                            |
    | `double`               | `c_double`                  | `f64`              |                                            |
    | `void*`                | `*mut c_void` / `*const c_void` | `*mut ()` / `*const ()` | `c_void` is an opaque type in Ferra        |
    | `const char*`          | `*const c_char`             | `*const i8`        | For C strings                              |
    | `bool` (`_Bool`)       | `c_bool`                    | `Bool`             |                                            |
    | `size_t`               | `c_size_t`                  | `usize`            |                                            |
    | `ptrdiff_t`            | `c_ptrdiff_t`               | `isize`            |                                            |
    | `intptr_t`             | `c_intptr_t`                | `isize`            |                                            |
    | `uintptr_t`            | `c_uintptr_t`               | `usize`            |                                            |
    | `FILE*` (example)      | `*mut CFile` / `*mut c_void` | `*mut ()`         | Opaque handle, `CFile` custom opaque struct |
    | `struct MyStruct`      | `MyStruct` (with `#[repr(C)]`) | `MyStruct`       | Ferra `data` class with `#[repr(C)]`      |
    | `enum MyEnum`          | `MyEnum` (with `#[repr(IntType)]`) | `MyEnum`         | Ferra `data`/`enum` with `#[repr(IntType)]` |
    | `RetType (*)(ArgTypes)` | `extern "C" fn(ArgTypes) -> RetType` | `fn(...)`   | Ferra function pointer type                |

*   **Primitive C Types**:
    *   **Integer Types**:
        *   `char`, `signed char`, `unsigned char`: Mapped to Ferra `i8` or `u8` (or specific `c_char`, `c_schar`, `c_uchar` type aliases if provided for exactness, which then alias to `i8`/`u8`).
        *   `short`, `unsigned short`: Mapped to Ferra `i16` or `u16` (or `c_short`, `c_ushort`).
        *   `int`, `unsigned int`: Mapped to Ferra `i32` or `u32` (or `c_int`, `c_uint`). This is a common default for `int` in C ABIs. *Note: Ferra's `Int` is 64-bit by default. For FFI with C `int`, a specific `i32` or `c_int` is preferred for clarity.*
        *   `long`, `unsigned long`: Mapped to Ferra `i32`/`u32` or `i64`/`u64` depending on the target platform's C ABI (e.g., `c_long`, `c_ulong` type aliases).
        *   `long long`, `unsigned long long`: Mapped to Ferra `i64` or `u64` (or `c_longlong`, `c_ulonglong`).
        *   C99 `stdint.h` types (e.g., `int32_t`, `uint64_t`): Direct mapping to Ferra's fixed-size integers (`i32`, `u64`, etc.). Ferra should provide type aliases for all standard C integer types (e.g., `c_int`, `c_size_t`, etc.) in a core FFI module.
    *   **Floating-Point Types**:
        *   `float`: Mapped to Ferra `f32` (or `c_float`).
        *   `double`: Mapped to Ferra `f64` (or `c_double`).
        *   `long double`: Mapping is platform-dependent and can be tricky. For v0.1, direct mapping might be to `f64` with a warning, or a specific `c_longdouble` type that could be an opaque struct if no direct Ferra equivalent exists on all platforms. Simpler to recommend avoiding `long double` in FFI signatures where possible for v0.1.
    *   **`void`**: C `void` as a return type maps to Ferra's `Unit` type `()` or simply no return type in the Ferra declaration if the function truly returns nothing. (e.g., `fn c_does_nothing();` or `fn c_returns_void() -> ();`).
    *   **C `bool` (`_Bool` from C99)**: Mapped to Ferra `Bool`. The representation (e.g., as a byte) must match the C ABI.

*   **Pointers**:
    *   `*const T` (C): Mapped to Ferra `*const CEquivOfT` (e.g., `*const c_int`).
    *   `*mut T` (C): Mapped to Ferra `*mut CEquivOfT` (e.g., `*mut c_int`).
    *   `void*` (C): Mapped to Ferra `*mut c_void` or `*const c_void` (where `c_void` is an opaque type alias representing C's `void` in pointer contexts).
    *   Ferra raw pointers (`*const T`, `*mut T`) are used. These pointers do not have Ferra's ownership or lifetime tracking; dereferencing them is an `unsafe` operation.

*   **C Strings (`char*`)**:
    *   `const char*` (for strings passed from C to Ferra, or Ferra to C as read-only): Mapped to Ferra `*const c_char` (or `*const i8`).
    *   `char*` (for mutable C strings, or strings Ferra needs to write into buffer provided by C): Mapped to Ferra `*mut c_char` (or `*mut i8`).
    *   **Null Termination**: Ferra code interacting with these must respect C's null-termination convention.
    *   **Conversion**: Helper functions will be needed in Ferra (likely in stdlib or an FFI utility module) to safely convert from `*const c_char` (and optionally a length) to a Ferra `String` (which involves copying and UTF-8 validation) and vice-versa if Ferra needs to prepare a C-compatible string. This is detailed further in Section 5.3.

*   **C Structs**:
    *   Mapped to Ferra `data` classes annotated with `#[repr(C)]` (or an equivalent attribute). This attribute ensures that the Ferra `data` class has a memory layout compatible with the C struct (e.g., field ordering, padding). The exact syntax and behavior of `#[repr(C)]` and other `repr` attributes are covered by TBD FFI-C-2 and should be formally defined in `SYNTAX_GRAMMAR_V0.1.md`.
        ```ferra
        // C struct:
        // struct CPoint { int x; int y; };

        // Ferra equivalent:
        #[repr(C)] // Attribute to ensure C-compatible layout
        data CPoint {
            x: c_int, // Assuming c_int is alias for i32 or platform's int
            y: c_int,
        }

        extern "C" {
            fn process_point(p: CPoint);
            fn get_point() -> CPoint;
            fn process_point_ptr(p_ptr: *const CPoint);
        }
        ```
    *   Passing by value vs. by pointer depends on the C function signature.

*   **C Enums**:
    *   Mapped to Ferra `data` types (potentially simple enums if Ferra adds them) also annotated with `#[repr(C_Enum)]` or `#[repr(IntType)]` to specify the underlying integer representation (e.g., `#[repr(c_int)]`). The specification for these representation attributes falls under TBD FFI-C-2 and requires grammar definition in `SYNTAX_GRAMMAR_V0.1.md`.
        ```ferra
        // C enum:
        // enum CColor { RED = 1, GREEN = 2, BLUE = 4 };

        // Ferra equivalent:
        #[repr(c_int)] // Ensures underlying type matches C enum's int
        data CColor { // Or `enum CColor { ... }` if Ferra gets C-like enums
            RED = 1,
            GREEN = 2,
            BLUE = 4,
        }
        ```
    *   The exact Ferra syntax for enums corresponding to C enums needs to align with Ferra's general enum design.

*   **Function Pointers (C)**:
    *   Mapped to Ferra's function pointer types, ensuring the calling convention is C-compatible.
    *   Syntax: `fn(CType1, CType2) -> CReturn` (within an `extern "C"` context for the type itself if it's complex, or a type alias).
        ```ferra
        // C: typedef int (*callback_t)(int, int);
        //    void register_callback(callback_t cb);

        // Ferra:
        type CCallback = extern "C" fn(c_int, c_int) -> c_int;

        extern "C" {
            fn register_callback(cb: CCallback);
        }
        ```

*   **Opaque Pointers**:
    *   C types that Ferra does not need to inspect internally (e.g., handles returned by a C library like `FILE*`) can be represented in Ferra as opaque structs/data classes holding a raw pointer, often `*mut c_void` or a newtyped pointer.
        ```ferra
        // C: typedef struct SomeOpaqueHandle* OpaqueHandle;
        //    OpaqueHandle create_handle();
        //    void process_with_handle(OpaqueHandle h);
        //    void destroy_handle(OpaqueHandle h);

        // Ferra:
        #[repr(transparent)] // If the struct is just a wrapper for the pointer
        data OpaqueHandle { ptr: *mut c_void } // Or just use *mut c_void directly

        extern "C" {
            fn create_handle() -> OpaqueHandle; // Or *mut c_void
            fn process_with_handle(h: OpaqueHandle);
            fn destroy_handle(h: OpaqueHandle);
        }
        ```
    *   This prevents Ferra from accidentally misinterpreting the C type's internal structure. All operations on such opaque types are done by calling further C functions.

*   **Type Aliases for Clarity**:
    *   It is highly recommended that Ferra provide a standard FFI module (e.g., `core::ffi` or `std::ffi`) that includes type aliases for common C types (e.g., `c_char`, `c_int`, `c_long`, `c_double`, `c_void`, `size_t`, `uintptr_t`).
    *   Using these aliases (`use core::ffi::*;`) in FFI declarations improves clarity and portability.
        ```ferra
        use core::ffi::{c_char, c_int, c_double}; // Example path

        extern "C" {
            fn c_calculate(input: c_int, scale: c_double) -> c_double;
            fn c_get_message() -> *const c_char;
        }
        ```
This detailed mapping is fundamental for correct and safe interaction with C libraries.

### 3.3. Calling Conventions

The calling convention defines how arguments are passed to functions (e.g., via registers or the stack), how return values are handled, and which registers are preserved by the caller or callee. For FFI, adhering to the correct calling convention is essential for interoperability.

*   **Default C Calling Convention**:
    *   When Ferra code calls a C function declared via `extern "C"`, Ferra will use the **default C calling convention** for the target platform and architecture.
    *   This is typically:
        *   **System V AMD64 ABI**: Used on Linux, macOS, BSD, and other Unix-like systems for x86-64. Arguments are passed primarily in registers (RDI, RSI, RDX, RCX, R8, R9 for integers/pointers; XMM0-XMM7 for floats/doubles), with additional arguments on the stack. Return values are typically in RAX (integers/pointers) or XMM0 (floats/doubles).
        *   **Microsoft x64 ABI**: Used on Windows for x86-64. The first four integer/pointer arguments are passed in RCX, RDX, R8, R9; floats/doubles in XMM0-XMM3. Additional arguments on the stack. Return values are in RAX or XMM0.
    *   The Ferra compiler's backend (e.g., LLVM backend as per `BACKEND_LLVM_X86-64.md`) will be responsible for generating code that adheres to these conventions.

*   **Specifying Other Calling Conventions (Future Consideration)**:
    *   While `extern "C"` implies the default C calling convention, some C libraries or OS APIs might use specific non-default conventions (e.g., `stdcall`, `fastcall` on older x86 systems, though less common on x86-64).
    *   For v0.1, Ferra will **only support the default C calling convention** for the target platform when using `extern "C"`.
    *   Support for explicitly specifying other calling conventions (e.g., `extern "stdcall" fn ...`) is a future consideration if strong use cases arise.

*   **Ferra's Internal Calling Convention**:
    *   Ferra's internal calling convention for Ferra-to-Ferra calls might be different and optimized for Ferra's features. However, this is not relevant for FFI with C. When an `extern "C"` boundary is crossed, the C ABI's calling convention is paramount.

Ensuring the compiler correctly implements the target's C calling convention is critical for FFI to function correctly.

### 3.4. Memory Management (Calling C from Ferra)

Managing memory across the FFI boundary between Ferra and C requires careful attention to ownership and lifetimes to prevent memory leaks, use-after-free errors, and double frees. Ferra's ownership system helps manage its own memory, but C operates under manual memory management rules.

*   **Responsibility for Memory Allocated by C**:
    *   If a C function allocates memory and returns a pointer to it (e.g., `char* result = c_strdup("hello");`), the C library's API will typically define who is responsible for freeing that memory and how (e.g., a corresponding `c_free_string(result)` function).
    *   Ferra code calling such C functions is responsible for ensuring that the C-allocated memory is eventually freed using the C library's designated deallocation function.
    *   This usually involves:
        1.  Storing the pointer returned from C.
        2.  Calling the C deallocation function via FFI when the memory is no longer needed.
        3.  This must be done within an `unsafe` block, as Ferra cannot automatically manage this C-allocated memory.
    *   Wrapper types in Ferra (e.g., a struct that holds the C pointer and implements Ferra's `Drop` trait - if/when Ferra has one analogous to Rust) can help automate this, calling the C free function when the Ferra wrapper goes out of scope. For v0.1, manual calls to C free functions will be the primary mechanism.

*   **Passing Ferra-Owned Data to C**:
    *   **Passing by Pointer (Borrowing from Ferra's perspective)**:
        *   When passing a pointer to Ferra-owned data to a C function (e.g., passing `*const c_char` from a Ferra `String` to a C function that only reads it), Ferra typically retains ownership.
        *   The Ferra data must be guaranteed to live at least as long as the C function call, and potentially longer if the C function stores the pointer for later use (this latter case is very dangerous and requires careful lifetime management or copying).
        *   Example:
            ```ferra
            extern "C" {
                fn c_print_string(s: *const c_char);
            }
            fn my_ferra_func() unsafe {
                let ferra_string = "Hello from Ferra"; // Ferra String (details of getting *const c_char TBD)
                let c_str_ptr = ferra_string.as_c_str_ptr(); // Conceptual: get a C-compatible pointer
                c_print_string(c_str_ptr);
                // `ferra_string` is still owned by Ferra and will be dropped by Ferra.
                // `c_str_ptr` must not be used by C after `ferra_string` is dropped.
            }
            ```
    *   **Transferring Ownership from Ferra to C**:
        *   This is less common and more complex. It implies that C code will take responsibility for freeing memory that was originally allocated by Ferra.
        *   This requires Ferra to provide a mechanism for C to deallocate Ferra-managed memory (e.g., by exporting a Ferra deallocation function callable from C), or for Ferra to allocate memory using C's allocator (e.g., `malloc`) if the data is to be fully managed by C.
        *   For v0.1, scenarios requiring transfer of Ferra-allocated memory ownership to C for deallocation by C should be approached with extreme caution and are generally discouraged in favor of C allocating and freeing its own memory, or Ferra passing borrows.
    *   **Copying Data**: Often the safest approach is to copy Ferra data into memory allocated by C (if C needs to own/modify it long-term) or into a temporary C-compatible buffer if C only needs transient access to a mutable copy.

*   **Pointers to Stack-Allocated Ferra Data**:
    *   Passing pointers to Ferra stack-allocated data to C functions is permissible but requires ensuring the C function does not retain the pointer beyond the lifetime of the Ferra stack frame. This is a common source of errors if not handled carefully.

*   **Ferra's `String` and `Vector<T>` to C**:
    *   **`String`**: To pass a Ferra `String` to a C function expecting `const char*`, Ferra needs to provide a null-terminated, UTF-8 (or other encoding if specified by C API) byte sequence. This usually involves:
        *   Getting a pointer to the string's internal buffer.
        *   Ensuring null termination (Ferra strings might not be null-terminated internally; a temporary null-terminated copy might be needed if the C function relies on it).
        *   The pointer is valid as long as the Ferra `String` is live and unmodified (if internal buffer pointer is used).
    *   **`Vector<T>` / Slices `[T]`**: Passed to C as a pointer to the first element and a length.
        *   `data_ptr: *const ElementType`
        *   `len: c_size_t`
        *   The C function must respect the provided length to avoid buffer overflows.
        *   If `T` is a Ferra `data` class, it must have `#[repr(C)]` for predictable layout.

*   **Guidance**:
    *   When a C API is unclear about memory ownership (e.g., for returned pointers or pointers passed in), assume the caller (Ferra) is responsible for managing any memory it allocates and for freeing memory returned by C if the C API documentation specifies this.
    *   Writing thin Ferra wrappers around C calls to manage these memory concerns is highly recommended to keep the `unsafe` FFI details localized.

Clear rules and developer diligence are paramount for memory safety when interfacing with C.

### 3.5. Error Handling (Calling C from Ferra)

C functions traditionally report errors in various ways, such as returning special values (e.g., -1, NULL), setting a global `errno` variable, or using specific error codes as return values. Ferra code calling C functions must be prepared to handle these C-style error conventions.

*   **Checking Return Codes**:
    *   Many C functions indicate errors via their return value. For example, a function returning a pointer might return `NULL` on error, or a function returning an integer might return `-1` or a non-zero value.
    *   Ferra code must check these return values.
        ```ferra
        extern "C" {
            fn c_might_fail_returns_int(input: c_int) -> c_int; // Returns -1 on error
            fn c_might_fail_returns_ptr(input: c_int) -> *mut c_void; // Returns NULL on error
        }

        fn call_c_functions() unsafe {
            let result_int = c_might_fail_returns_int(10);
            if result_int == -1 {
                // Handle error, potentially based on errno or other C API specifics
                let current_errno = get_c_errno(); // Conceptual helper
                eprintln("C function failed with errno: " + String::from_int(current_errno));
                // return Err(MyError::from_errno(current_errno)); // If wrapping in Ferra Result
            } else {
                // Process result_int
            }

            let result_ptr = c_might_fail_returns_ptr(20);
            if result_ptr.is_null() { // Assuming a helper `is_null()` for raw pointers
                // Handle error
                eprintln("C function returned NULL pointer.");
                // return Err(MyError::NullPtrReturned);
            } else {
                // Process result_ptr (remembering to free it if C API requires)
            }
        }
        ```

*   **Interpreting `errno`**:
    *   Some C functions, particularly POSIX-compliant ones, report errors by setting the global `errno` variable (defined in `<errno.h>`). `errno` is only valid immediately after a C function call indicates an error (e.g., returns -1).
    *   Ferra will need a way to access `errno`. This could be:
        *   An FFI call to a C helper function that returns the current thread-local `errno` value.
        *   A compiler intrinsic or a stdlib function `core::ffi::get_errno() -> c_int`.
    *   The value of `errno` can then be used to determine the specific error (e.g., by comparing against constants like `EINVAL`, `ENOENT` from C headers, which Ferra might also provide aliases for in its FFI module).

*   **Other C Error Mechanisms**:
    *   Some C libraries have their own dedicated error handling functions (e.g., `library_get_last_error_code()`, `library_get_error_string(code)`).
    *   Ferra code would need to call these additional C functions via FFI to retrieve detailed error information.

*   **Wrapping C Errors in Ferra `Result<T, E>`**:
    *   It's good practice to wrap FFI calls that can fail into safer Ferra functions that return `Result<T, ErrorType>`, where `ErrorType` is a Ferra enum or struct representing the possible C errors.
        ```ferra
        // Conceptual Ferra error type
        data CFfiError {
            code: c_int,     // e.g., errno value or specific C error code
            message: String,
        }

        fn safe_c_call_wrapper(input: c_int) -> Result<c_int, CFfiError> unsafe {
            let ret = c_might_fail_returns_int(input);
            if ret == -1 {
                let err_code = core::ffi::get_errno(); // Conceptual
                return Err(CFfiError {
                    code: err_code,
                    message: get_c_error_string(err_code), // Conceptual helper
                });
            }
            return Ok(ret);
        }
        ```

*   **No Exception Handling**:
    *   C does not have exceptions in the style of C++ or Java. Ferra should not expect to catch C++ exceptions through a C FFI boundary unless very specific platform/compiler mechanisms are used (out of scope for v0.1). Errors are signaled as described above.

Handling errors robustly is crucial for stable FFI. Ferra developers must consult the documentation of the C libraries they are calling to understand their specific error reporting conventions.

### 3.6. Safety Considerations (Calling C from Ferra)

Interacting with C code via FFI introduces operations that the Ferra compiler cannot statically verify for safety. Therefore, these operations must occur within an `unsafe` context in Ferra, and the developer assumes responsibility for upholding safety invariants.

*   **The `unsafe` Context**:
    *   All direct calls to functions declared in an `extern "C"` block must be wrapped in an `unsafe { ... }` block or be part of an `unsafe fn`.
    *   Dereferencing raw pointers (e.g., `*const c_char`, `*mut MyCStruct`) obtained from C or created to pass to C is an `unsafe` operation.
    *   The `unsafe` block signals that the operations within it might violate Ferra's usual safety guarantees if not handled correctly by the programmer.

*   **Potential Hazards from C Side**:
    *   **Null Pointers**: C functions might return `NULL` pointers where a valid pointer is expected, or accept pointers that could be `NULL`. Ferra code must diligently check for `NULL` before dereferencing pointers from C, unless the C API explicitly guarantees non-null return.
    *   **Buffer Overflows/Overreads**: C functions operating on buffers (e.g., strings, arrays) often rely on the caller to provide correct buffer sizes or ensure null termination. Passing incorrectly sized buffers or non-null-terminated strings from Ferra can lead to C-side buffer overflows or reads. Conversely, C functions might write beyond buffer boundaries if not implemented carefully.
    *   **Data Races**: If the C code is not thread-safe and is called from a concurrent Ferra context (though Ferra's v0.1 concurrency is deterministic actors, future multithreading could be an issue), data races can occur if the C code accesses shared mutable state without proper synchronization. Ferra's safety guarantees do not extend into the C code.
    *   **Incorrect Type Assumptions**: If the Ferra declaration of a C function mismatches the actual C signature (e.g., wrong parameter types, incorrect return type), it will lead to undefined behavior.
    *   **Resource Leaks**: C libraries often require explicit resource deallocation (e.g., `free`, custom cleanup functions). Failure to call these from Ferra for resources obtained from C will lead to leaks.
    *   **Invalid State/Invariants**: C functions may have implicit preconditions or expect global state to be configured in a certain way. Calling them without satisfying these can lead to errors or crashes.

*   **Best Practices for Ferra FFI Wrappers**:
    *   **Minimize `unsafe` Scope**: Keep `unsafe` blocks as small as possible, ideally just around the direct FFI call or raw pointer dereference.
    *   **Input Validation**: Before calling a C function, validate inputs from the Ferra side where possible (e.g., check that pointers passed to C are not null if the C function doesn't expect them, ensure string encodings are correct).
    *   **Output Validation/Conversion**: When receiving data from C (especially pointers or raw data buffers), validate it (e.g., check for `NULL`, check lengths) and convert it into safe Ferra types (e.g., convert `*const c_char` to Ferra `String`, copying the data).
    *   **Error Handling**: As discussed in Section 3.5, translate C error codes/`errno` into Ferra `Result` types.
    *   **Resource Management**: Implement Ferra's `Drop` trait (once available) for Ferra structs that wrap C resources, ensuring the C cleanup function is called automatically when the Ferra wrapper goes out of scope.
    *   **Documentation**: Clearly document the assumptions made about the C API, especially regarding memory ownership, lifetimes, nullability of pointers, and error handling, within the FFI wrapper code.

By being aware of these potential issues and adopting careful wrapping practices, developers can significantly mitigate the risks associated with FFI.

## 4. Calling Ferra from C/C++

This section details how Ferra functions and data structures can be exposed and made callable from external C and C++ code. The primary mechanism is to provide a C-compatible ABI.

### 4.1. Exporting Ferra Functions

For a Ferra function to be callable from C (and thus C++ via an `extern "C"` declaration on the C++ side), it needs to be explicitly marked for export with a C ABI. This involves controlling its visibility, calling convention, and symbol name in the compiled library.

*   **Core Requirements for Exporting**:
    1.  **C-Compatible Signature**: The function's parameters and return type must use FFI-safe types (see Section 4.2).
    2.  **C Calling Convention**: The function must be compiled to use the target platform's C calling convention.
    3.  **Stable Symbol Name**: The function must have a predictable, unmangled symbol name in the object file/library.
    4.  **Public Visibility**: The function must be declared `pub` in Ferra.

*   **Syntax for Exporting**:
    *   Ferra uses a combination of `pub`, the `extern "C"` keyword to specify the ABI, and attributes to control symbol naming.
    *   **Conceptual Syntax**:
        ```ferra
        #[no_mangle] // Ensures the function name is used as the symbol, preventing Ferra name mangling.
        pub extern "C" fn exported_ferra_function(arg1: c_int, arg2: *const c_char) -> c_int {
            // Function body...
            // Ensure types used (c_int, *const c_char) are C-compatible.
            // Be mindful of panic handling (see Section 4.4).
            return 0; // Example return
        }

        // Another example with a potentially different exported symbol name
        #[export_name = "my_c_api_calculate_sum"] // Specifies the exact C symbol name.
                                                 // #[no_mangle] might be implied or still good practice.
        pub extern "C" fn ferra_calculate_sum(a: i32, b: i32) -> i32 {
            return a + b;
        }
        ```
    *   `pub`: Makes the Ferra function visible outside its module.
    *   `extern "C"`: Instructs the compiler to use the C ABI for this function, including the platform's C calling convention.
    *   **Attributes for Symbol Naming (Summary Table)**:
        | Attribute                 | Purpose                                                                                                | Notes                                                                 |
        |---------------------------|--------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------|
        | `#[no_mangle]`            | Prevents Ferra's default name mangling, using the Ferra function identifier as the base symbol name.   | Essential for predictable C linkage.                                |
        | `#[export_name = "name"]` | Exports the function with the exact C symbol `name` (a string literal). Overrides `#[no_mangle]`'s default. | Useful for matching existing C APIs or specific naming requirements.    |
        *(The general attribute syntax is defined in `SYNTAX_GRAMMAR_V0.1.md` Section 1.3.5. TBD FFI-C-1 covers final exact syntax and interaction of these attributes, including `#[no_mangle]` and `#[export_name]`, which should be formally specified in relation to the grammar document.)*

*   **Example C Header Declaration (Conceptual)**:
    When Ferra exports functions, C/C++ code consuming the Ferra library would typically use header declarations like the following. These might be manually written or auto-generated (see Section 4.6). The `FFI_EXPORT` macro handles platform-specific export/visibility.
    ```c
    // Conceptual C header (e.g., my_ferra_library.h)
    #ifndef MY_FERRA_LIBRARY_H
    #define MY_FERRA_LIBRARY_H

    // Standard macro for handling symbol visibility across platforms
    #if defined(_WIN32) || defined(__CYGWIN__)
    #  if defined(MY_FERRA_LIB_BUILDING_DLL) // Defined when building the DLL
    #    define FFI_EXPORT __declspec(dllexport)
    #  elif defined(MY_FERRA_LIB_USING_DLL)    // Defined when linking against the DLL
    #    define FFI_EXPORT __declspec(dllimport)
    #  else
    #    define FFI_EXPORT // Static library or executable
    #  endif
    #else // Non-Windows platforms (GCC/Clang)
    #  if __GNUC__ >= 4
    #    define FFI_EXPORT __attribute__((visibility("default")))
    #  else
    #    define FFI_EXPORT
    #  endif
    #endif

    #ifdef __cplusplus
    extern "C" {
    #endif

    FFI_EXPORT int exported_ferra_function(int arg1, const char* arg2);
    FFI_EXPORT int32_t my_c_api_calculate_sum(int32_t a, int32_t b);

    #ifdef __cplusplus
    } // extern "C"
    #endif

    #endif // MY_FERRA_LIBRARY_H
    ```
    *(The `FFI_EXPORT` macro, shown above, is a common pattern. `MY_FERRA_LIB_BUILDING_DLL` and `MY_FERRA_LIB_USING_DLL` would be defined by the build system depending on whether the Ferra library is being built as a DLL or linked as a DLL. For static libraries, `FFI_EXPORT` often resolves to nothing.)*

*   **Name Mangling Considerations**:
    *   Without `#[no_mangle]` or `#[export_name]`, Ferra functions would typically be name-mangled by the compiler to support features like overloading (if Ferra supports it) and to avoid name collisions between different modules or generic instantiations. This mangled name is not suitable for direct C FFI.
    *   The combination of `pub extern "C"` and appropriate symbol naming attributes is the standard way to create a stable, predictable C ABI.

*   **Limitations for Exported Functions**:
    *   **FFI-Safe Types**: All parameter types and the return type of an exported Ferra function must be FFI-safe types that have a well-defined C ABI representation (see Section 4.2).
    *   **No Direct Support for Ferra Generics in Exported Signature**: While a generic Ferra function can be *implemented* and then monomorphized instances of it exported with concrete C-compatible types, the `extern "C"` signature itself cannot be generic in a way C understands.
        ```ferra
        // Ferra generic function
        fn add<T: Numeric>(a: T, b: T) -> T { return a + b; }

        // Exported instance for i32
        #[no_mangle]
        pub extern "C" fn add_i32(a: i32, b: i32) -> i32 {
            return add(a, b); // Calls the generic Ferra function
        }
        ```
    *   **Panic Handling**: Ferra panics should not be allowed to unwind across the FFI boundary into C code, as C is generally not prepared to handle them. This is discussed in Section 4.4.

By using these conventions, Ferra can expose a stable and callable C API.

### 4.2. Data Type Mapping (Ferra -> C)

When Ferra functions are exported with a C ABI, their Ferra parameter and return types must be mapped to types that C can understand and that have a compatible representation. This is largely the reverse of the mapping in Section 3.2.

*   **Ferra Primitive Types to C**:
    *   Ferra `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`: Mapped to corresponding standard C integer types (e.g., `int8_t`, `uint32_t`, `long long` etc., or via type aliases like `c_int`).
    *   Ferra `f32`, `f64`: Mapped to C `float`, `double`.
    *   Ferra `Bool`: Mapped to C `bool` (`_Bool`) or a compatible integer type (e.g., `unsigned char` or `int`) as per the target C ABI for booleans.
    *   Ferra `Char` (Unicode scalar): Typically passed as a C `int` or `long` (e.g., `wchar_t` or `uint32_t` if representing UTF-32) if C needs to interpret it as a character. If just passing the bits, a `uint32_t` is common.
    *   Ferra `Unit` (`()`) as a return type: Maps to C `void` return. Ferra functions returning `Unit` effectively become `void` functions in C.

*   **Raw Pointers from Ferra**:
    *   Ferra `*const T` and `*mut T` (where `T` is an FFI-safe Ferra type corresponding to `CType`): These can be directly passed to C as `const CType*` and `CType*` respectively. C code receiving these pointers must respect their constness and understand the lifetime/validity of the pointed-to data (managed by Ferra, unless ownership is explicitly transferred).

*   **Exposing Ferra `String` to C**:
    *   A Ferra `String` cannot be directly passed as a `char*` that C can treat like a typical C string without careful handling, because Ferra `String`s:
        *   Are UTF-8.
        *   Are not necessarily null-terminated internally.
        *   Manage their own memory.
    *   **Common Patterns**:
        1.  **Pass as `(*const c_char, c_size_t)` pair**: Ferra exports a function that returns a pointer to the string's UTF-8 byte buffer and its length. The C caller receives both and must not assume null termination if not explicitly guaranteed. The pointer is valid as long as the Ferra `String` (or the underlying data) is live.
            ```ferra
            // Ferra side
            #[no_mangle]
            pub extern "C" fn get_ferra_string_ptr(s: &String) -> *const c_char { // Assuming String has .ptr()
                return s.ptr(); // Conceptual
            }
            #[no_mangle]
            pub extern "C" fn get_ferra_string_len(s: &String) -> c_size_t { // Assuming String has .len()
                return s.len(); // Byte length
            }
            ```
        2.  **Copy to a C-managed buffer**: Ferra provides a function that copies the string content (optionally null-terminated) into a buffer provided by the C caller. The C caller manages the buffer's memory.
        3.  **Return a C-allocatable string**: Ferra allocates a `char*` using a C-compatible allocator (e.g., `malloc` via FFI), copies the Ferra string content into it (with null termination), and transfers ownership of this `char*` to C. C is then responsible for calling `free` on it.
    *   The choice depends on the desired memory management model and performance trade-offs. Pattern 1 (pointer + length to Ferra's buffer) is often efficient for read-only access but requires C to be careful about lifetimes.

*   **Exposing Ferra `data` Classes to C**:
    *   Ferra `data` classes intended for C interop must be annotated with `#[repr(C)]` to ensure a C-compatible memory layout.
    *   **Passing by Value**: If the `data` class is FFI-safe and C-compatible (all fields are FFI-safe), it can be passed by value or returned by value, and C will see it as a corresponding C struct.
    *   **Passing by Pointer**: More commonly, a pointer to a Ferra `data` instance (`*const MyData` or `*mut MyData`) is passed to C. C receives a pointer to the struct. Ferra still owns the data unless ownership is explicitly transferred (which is complex).
        ```ferra
        #[repr(C)]
        pub data MyPoint { x: f64, y: f64 }

        #[no_mangle]
        pub extern "C" fn process_ferra_point(p: MyPoint) { /* process copy */ }

        #[no_mangle]
        pub extern "C" fn process_ferra_point_by_ref(p_ptr: *const MyPoint) unsafe {
            // C can read p_ptr->x, p_ptr->y
        }
        ```

*   **Passing Ferra `Vector<T>` or Slices `[T]` to C**:
    *   Similar to strings, these are typically passed as a pointer to the first element and a length.
    *   `data_ptr: *const ElementCType` (or `*mut` if C can modify)
    *   `len: c_size_t`
    *   The elements of type `T` must themselves be FFI-safe and map to `ElementCType`. If `T` is a Ferra `data` class, it needs `#[repr(C)]`.
    *   C code must not access beyond `len` elements. Ferra owns the vector's memory.

*   **Opaque Handles for Ferra Objects**:
    *   If a Ferra object's internal structure should not be exposed to C, or if it's not directly C-compatible (e.g., contains complex Ferra-specific types, closures), it can be passed to C as an opaque handle.
    *   This is typically a raw pointer (`*mut c_void` or a newtyped pointer representing the Ferra object's address).
    *   C code treats this as an opaque type and can only pass it back to other exported Ferra functions that know how to operate on it.
    *   Ferra remains responsible for the actual object's lifetime and memory management.
        ```ferra
        // In Ferra:
        // pub data ComplexObject { ... } // Internal Ferra structure
        // fn new_complex_object() -> *mut ComplexObject { ... } // Returns heap-allocated
        // fn process_complex_object(obj_ptr: *mut ComplexObject) { ... }
        // fn free_complex_object(obj_ptr: *mut ComplexObject) { ... }


        #[no_mangle]
        pub extern "C" fn create_my_object() -> *mut c_void { // C sees opaque handle
            let obj = new_complex_object_internal(); // Assume this creates and boxes the object
            return obj as *mut c_void; // Cast Ferra pointer to opaque C pointer
        }

        #[no_mangle]
        pub extern "C" fn operate_on_my_object(handle: *mut c_void) unsafe {
            let obj_ptr = handle as *mut ComplexObjectInternal; // Cast back in Ferra
            // use obj_ptr
        }

        #[no_mangle]
        pub extern "C" fn destroy_my_object(handle: *mut c_void) unsafe {
            let obj_ptr = handle as *mut ComplexObjectInternal;
            // deallocate/drop obj_ptr correctly
        }
        ```
    *   This pattern is common for exposing object-oriented-like APIs to C.

Careful definition of these mappings is essential for creating a usable and correct C API from Ferra.

### 4.3. Memory Management (Calling Ferra from C/C++)

When C/C++ code calls Ferra functions, memory management responsibilities for data passed across the boundary must be clear. Ferra's ownership system manages its own heap, which is distinct from memory managed by C's `malloc`/`free`.

*   **Rules for Memory Passed from Ferra to C**:
    *   **Ferra Retains Ownership (Typically)**: If a Ferra function returns a pointer to Ferra-managed memory (e.g., a pointer obtained from a Ferra `String` or `Vector<T>`), Ferra typically retains ownership.
        *   The C code receives a raw pointer that is essentially a borrow.
        *   The C code **must not** attempt to `free()` this pointer using C's `free`, as it was not allocated by C's `malloc` family.
        *   The C code must ensure it does not use this pointer beyond the lifetime of the underlying Ferra data. This is the most challenging aspect from the C side, as C has no equivalent of Ferra's borrow checker.
        *   To manage this, Ferra often needs to export explicit deallocation functions for any Ferra-allocated resources it hands out as opaque handles or managed pointers (see below).
    *   **Transferring Ownership to C (Rare, Complex)**: If Ferra needs to give C full ownership of some data (including the responsibility to free it), Ferra would have to:
        1.  Allocate the memory using a C-compatible allocator (e.g., by calling `malloc` via FFI).
        2.  Copy the data into this C-allocated buffer.
        3.  Return the pointer to C. C would then be responsible for calling `free` on it.
        This pattern is more for Ferra *providing* data to a C system that expects to own it.

*   **Functions for C to Allocate/Deallocate Ferra Objects/Memory**:
    *   If C code needs to work with Ferra objects over a longer term, or if Ferra objects are complex and their creation/destruction involves Ferra-specific logic (like running Ferra `Drop` implementations), it's best practice for Ferra to export C-callable functions for managing these objects.
    *   **Creation Functions**:
        ```ferra
        // Ferra code
        #[repr(C)]
        pub data MyFerraObject { id: i32, /* other fields */ }

        #[no_mangle]
        pub extern "C" fn ferra_my_object_create(id_val: i32) -> *mut MyFerraObject {
            let new_obj = MyFerraObject { id: id_val, /* ... */ };
            // Allocate on Ferra's heap, box it, and return a raw pointer.
            // The returned pointer is now "owned" conceptually by the C caller
            // in the sense that C must eventually call ferra_my_object_destroy.
            let boxed_obj = Box::new(new_obj); // Conceptual Ferra boxing
            return Box::into_raw(boxed_obj);
        }
        ```
    *   **Deallocation/Destroy Functions**:
        ```ferra
        #[no_mangle]
        pub extern "C" fn ferra_my_object_destroy(obj_ptr: *mut MyFerraObject) {
            if obj_ptr.is_null() {
                return;
            }
            unsafe {
                // Convert raw pointer back to a Box to let Ferra's memory manager
                // deallocate it and run any Drop implementations.
                let _boxed_obj_to_drop = Box::from_raw(obj_ptr);
            }
        }
        ```
    *   C code would then use these:
        ```c
        // C code
        // MyFerraObject* obj = ferra_my_object_create(123);
        // if (obj) {
        //     // use obj via other exported Ferra functions that take MyFerraObject*
        //     ferra_my_object_destroy(obj);
        // }
        ```
    *   This "create/destroy" pattern for opaque types or complex Ferra objects passed to C is common and helps encapsulate Ferra's memory management.

*   **Strings and Collections Returned to C**:
    *   If a Ferra function needs to return a string or a collection to C, it should generally follow one of these patterns:
        1.  **C-Allocated Buffer (Caller-Provided)**: The C caller provides a buffer and its size. The Ferra function writes into this buffer. This is safe as C manages the memory.
            ```ferra
            // Ferra function
            #[no_mangle]
            pub extern "C" fn get_string_into_c_buffer(buffer: *mut c_char, buffer_len: c_size_t) -> c_int {
                let ferra_str = "Hello C!";
                // Safely copy ferra_str into buffer, respecting buffer_len, null-terminate.
                // Return length written (excluding null) or error code.
                // ... (implementation details) ...
                return 0; // success
            }
            ```
        2.  **Ferra-Allocated, C-Freeable**: Ferra allocates memory using a C-compatible allocator (e.g., `malloc` via FFI), copies its string/data into it, and returns the pointer. C is responsible for calling `free`.
        3.  **Pointer to Ferra's Internal Data with Deallocator**: Ferra returns a pointer to its internal data (e.g., `String` buffer) along with a dedicated function that C *must* call to deallocate/release that specific Ferra resource.
            ```ferra
            // Ferra String handle (conceptual, not necessarily a raw *c_char)
            // type FerraStringHandle = *mut c_void; // Opaque handle

            // #[no_mangle]
            // pub extern "C" fn ferra_create_string_for_c(contents: *const c_char) -> FerraStringHandle;
            // #[no_mangle]
            // pub extern "C" fn ferra_get_string_chars(handle: FerraStringHandle) -> *const c_char;
            // #[no_mangle]
            // pub extern "C" fn ferra_free_string_handle(handle: FerraStringHandle);
            ```

*   **General Principle**: Avoid ambiguity. The API contract between Ferra and C must be crystal clear about who allocates memory, who owns it, and who is responsible for freeing it. When in doubt, prefer patterns where each side manages memory it allocates.

### 4.4. Error Handling (Ferra -> C)

When C/C++ code calls an exported Ferra function, Ferra needs a way to communicate success or failure back to the C caller in a C-idiomatic way. Ferra's internal error handling (e.g., `Result<T,E>`, panics) must be translated at the FFI boundary.

*   **Primary Mechanism: Return Codes**:
    *   Exported Ferra functions should primarily signal errors to C callers using return codes. This is the most common and understood error handling pattern in C.
    *   A common convention is for the function to return an integer status code:
        *   `0` often indicates success.
        *   Non-zero values (e.g., specific positive integers or -1) indicate different types of errors. These error codes should be documented as part of the C API.
    *   If the Ferra function naturally returns a value (e.g., a pointer or a calculated number), a special value within that type's range (e.g., `NULL` for pointers, `-1` for integers that are normally non-negative) can be used to indicate an error, with more detailed error information potentially available through other means.

*   **Out-Parameters for Error Details**:
    *   To provide more detailed error information beyond a simple status code, "out-parameters" (pointers to C types that the Ferra function writes into) can be used.
    *   Example:
        ```ferra
        // Ferra function signature
        #[no_mangle]
        pub extern "C" fn ferra_do_operation(
            input: c_int,
            out_result_data: *mut c_int, // Pointer to store actual result data on success
            out_error_code: *mut c_int   // Pointer to store a specific error code on failure
        ) -> c_int { // Returns 0 for success, non-0 for general failure
            // ... Ferra logic ...
            // if success:
            //     unsafe { *out_result_data = ferra_result; }
            //     unsafe { *out_error_code = 0; } // Or some success code
            //     return 0; // Success
            // if failure:
            //     unsafe { *out_error_code = specific_ferra_error_to_c_code(ferra_err); }
            //     return -1; // General failure
            return 0; // Placeholder
        }
        ```
    *   The C caller would pass pointers to variables where Ferra can write detailed error codes or even pointers to error message strings (which must then be managed carefully regarding memory, see Section 4.3).

*   **Representing Ferra `Result<T, E>` to C**:
    *   Ferra's `Result<T, E>` type is not directly translatable to a standard C type.
    *   When an exported Ferra function internally produces a `Result`:
        1.  On `Ok(value)`:
            *   If `T` is `Unit`, the C function can return `0` (success).
            *   If `T` is another FFI-safe type, the C function can return the C-compatible representation of `value`. If the return channel is already used for this, error indication might be solely through a status code or an out-parameter.
        2.  On `Err(error_value)`:
            *   The C function should return a C error indicator (e.g., a non-zero status code, `NULL` if it returns a pointer).
            *   Optionally, it can convert `error_value` into a C-understandable error code and/or message string and pass it back via out-parameters.
            ```ferra
            // Conceptual internal Ferra error
            data MyFerraError { code: i32, description: String }

            // Ferra function returning Result internally
            fn do_something_fallible(input: i32) -> Result<i32, MyFerraError> {
                if input < 0 {
                    return Err(MyFerraError { code: 1, description: "Input negative" });
                }
                return Ok(input * 2);
            }

            #[no_mangle]
            pub extern "C" fn ferra_wrapper_for_fallible(
                input: c_int,
                out_value: *mut c_int // For Ok result
            ) -> c_int { // 0 for Ok, error code for Err
                match do_something_fallible(input) {
                    Ok(val) => unsafe {
                        *out_value = val;
                        return 0; // Success
                    },
                    Err(err) => unsafe {
                        // Optionally, pass more details of `err` via other out-parameters
                        // For now, just return the error code from MyFerraError
                        return err.code; // Return Ferra error code directly as C int
                    }
                }
            }
            ```

*   **Panic Handling Across FFI Boundary (Crucial)**:
    *   Ferra panics **MUST NOT** be allowed to unwind across the FFI boundary into C/C++ code. C/C++ is generally not prepared to handle Ferra's panic unwinding mechanism (which might involve different stack unwinding libraries, destructors, etc.). This would likely lead to undefined behavior or a crash.
    *   **Solution**: Every `pub extern "C" fn` exposed by Ferra must internally catch any potential panics that occur within its execution (including calls to other Ferra functions).
        *   If a panic is caught:
            1.  The function must immediately stop further execution of normal Ferra logic.
            2.  It should return a special, documented C error code that indicates a "fatal Ferra panic" or "unrecoverable internal error" occurred. This alerts the C caller that something went seriously wrong within the Ferra component.
            3.  It should avoid attempting to return complex data or call further Ferra code that might also panic. The state of the Ferra runtime might be compromised.
            4.  Logging the panic details on the Ferra side (e.g., to stderr or a log file) before returning to C is highly advisable for debugging.
        *   This effectively means wrapping the body of each `extern "C"` Ferra function in a mechanism equivalent to Rust's `std::panic::catch_unwind` (if Ferra develops such a feature) or a similar construct that can trap panics.
        *   (This addresses TBD FFI-C-4: Panic unwinding strategy).
        *   Example (conceptual):
            ```ferra
            // Standard C error code to indicate a panic occurred in Ferra
            const C_ERR_FERRA_PANIC: c_int = -99; // Or some other agreed-upon value

            #[no_mangle]
            pub extern "C" fn potentially_panicking_ferra_op(input: c_int) -> c_int {
                // Conceptual: Ferra's equivalent of catch_unwind
                let result = ferra_runtime::catch_panic(|| {
                    // Original Ferra logic that might panic
                    if input == 0 {
                        panic("Input cannot be zero!");
                    }
                    return input * 10; // Normal result
                });

                match result {
                    Ok(value) => return value, // Normal return
                    Err(_panic_payload) => {
                        // Log the panic on Ferra side (e.g., to stderr)
                        eprintln_ferra_internal("Panic occurred in exported Ferra function!");
                        return C_ERR_FERRA_PANIC; // Return specific error code to C
                    }
                }
            }
            ```
    *   The C caller, upon receiving `C_ERR_FERRA_PANIC`, should understand that the Ferra component is likely in an unstable state and should probably not be called again within the current process, or the process might need to terminate.

Clear and consistent error reporting from Ferra to C is vital for building robust interoperable systems.

### 4.5. Callbacks: Passing Ferra Functions to C

A common FFI scenario involves C libraries that require function pointers as callbacks for events, custom processing, or iteration. Ferra must provide a way to pass its functions to C for such purposes.

*   **Use Case**:
    *   Registering event handlers with a C GUI library.
    *   Providing a custom comparison function to a C sorting algorithm.
    *   Iterating over a C data structure using a Ferra function for each element.

*   **Mechanism: C-Compatible Function Pointers**:
    *   To be callable from C, a Ferra function passed as a callback must be (or be wrapped by) an `extern "C"` function with FFI-safe parameter and return types.
    *   The C library will expect a simple function pointer that adheres to the C calling convention.

*   **Callback Variants Summary**:

    | Scenario                        | Ferra Function Type                                     | C Signature Example                    | `userdata` Pattern | Notes                                                                     |
    |---------------------------------|---------------------------------------------------------|----------------------------------------|--------------------|---------------------------------------------------------------------------|
    | Stateless Ferra Function        | `pub extern "C" fn(...) -> ...`                           | `void (*cb)(int)`                      | Optional           | Direct pointer to Ferra function.                                         |
    | Stateful Ferra Closure/Context  | Static `extern "C" fn` (trampoline) + Ferra closure data | `void (*cb)(int, void* ctx)`           | Required           | Trampoline casts `void* ctx` to Ferra context and calls Ferra closure.    |

*   **Ferra Functions Suitable as Direct Callbacks (Stateless Example)**:
    *   A top-level Ferra function already declared as `pub extern "C" fn_name(arg_types...) -> ret_type { ... }` can typically be passed directly. Its address is a C-compatible function pointer.
        ```ferra
        // Ferra side
        #[no_mangle] // Not strictly needed for the callback itself, but good if it's also exported
        pub extern "C" fn my_ferra_callback(data: c_int, context: *mut c_void) -> c_int {
            println("Ferra callback invoked with data: " + String::from_int(data));
            // Process context if needed (unsafe cast)
            return data * 2;
        }

        // Declaration of C function expecting a callback
        extern "C" {
            // C signature: typedef int (*my_c_callback_t)(int data, void* user_context);
            //              void register_c_callback(my_c_callback_t cb, void* user_context);
            fn register_c_callback(cb: extern "C" fn(c_int, *mut c_void) -> c_int, user_context: *mut c_void);
        }

        fn setup_callback() unsafe {
            let my_context_data: *mut c_void = core::ptr::null_mut(); // Example: no context or prepare context
            register_c_callback(my_ferra_callback, my_context_data);
        }
        ```

*   **Ferra Closures as Callbacks (Complexities and Solutions)**:
    *   Ferra closures (if/when fully specified with environment capture) present a challenge because a standard C function pointer cannot directly represent a closure that captures its environment (which often makes the closure a "fat pointer" containing both the function address and a pointer to the captured data).
    *   **Stateless Closures**: If a Ferra closure captures no external environment, it might be directly convertible to an `extern "C"` function pointer.
    *   **Stateful Closures (Trampoline Pattern)**: For closures that capture Ferra state, the common solution is the "trampoline" pattern, often used in conjunction with a `void* userdata` argument provided by the C API:
        1.  **Userdata**: The C API for registering a callback usually accepts a `void* userdata` (or `void* context`) parameter, which it will pass back to the callback function each time it's invoked.
        2.  **Ferra Context**: Ferra code allocates its closure's context data (or a handle to it, e.g., a `Box` containing the closure object) and casts its address to `*mut c_void` to be passed as `userdata`.
        3.  **Static Trampoline Function**: A static, top-level Ferra function is defined with an `extern "C"` signature matching what the C library expects. This is the "trampoline."
        4.  **Registration**: Ferra registers the *trampoline function's pointer* with the C library, along with the `*mut c_void` pointer to Ferra's closure context.
        5.  **Invocation**: When the C library invokes the callback, it calls the trampoline, passing the original `userdata`.
        6.  **Trampoline Logic**: Inside the Ferra trampoline function:
            *   It receives the `userdata` pointer.
            *   It `unsafe`ly casts this `*mut c_void` back to the actual Ferra closure/context type.
            *   It then invokes the Ferra closure, passing the necessary arguments (and the reconstituted context).
            *   It handles any return value from the closure and converts it to the C-expected return type.
            *   Crucially, it must catch any panics from the Ferra closure (see Section 4.4).

        ```ferra
        // Conceptual Ferra closure (syntax illustrative)
        // let my_closure = |value: c_int, captured_factor: &Int| -> c_int {
        //     println("Closure called with " + String::from_int(value));
        //     return value * (*captured_factor);
        // };

        // Assume we have a way to box this closure and its captured data
        // struct FerraCallbackContext {
        //     closure_fn: ???, // Representation of the callable closure
        //     captured_data_ptr: *const Int,
        // }

        // Trampoline function
        extern "C" fn ferra_trampoline_for_c(data: c_int, user_context: *mut c_void) -> c_int {
            unsafe {
                if user_context.is_null() { return -1; } // Or some error
                
                // Conceptual: Cast user_context back to Ferra's context/closure type
                // let context = user_context as *const FerraCallbackContext; 
                // let factor = *((*context).captured_data_ptr);
                // let result = ((*context).closure_fn)(data, factor); // Invoke actual closure

                // Simplified for illustration: assume user_context is just a pointer to an Int factor
                let factor_ptr = user_context as *const Int;
                let factor = *factor_ptr;
                // println("Trampoline: data=" + String::from_int(data) + ", factor=" + String::from_int(factor));
                
                // Here, one would call the actual Ferra closure logic.
                // This example just does a simple operation.
                // Ensure to catch panics from the real closure call.
                let panic_result = ferra_runtime::catch_panic(|| {
                     data * factor // Placeholder for actual closure call
                });

                match panic_result {
                    Ok(val) => return val,
                    Err(_) => {
                        // eprintln_ferra_internal("Panic in Ferra callback trampoline!");
                        return -99; // C_ERR_FERRA_PANIC equivalent
                    }
                }
            }
        }

        fn setup_closure_callback() unsafe {
            let captured_value: Int = 5;
            // Prepare context: C will own this pointer for duration of callback, Ferra must ensure it's valid.
            // If captured_value is on stack, this is dangerous unless its lifetime is guaranteed.
            // Better: Box it or use a handle that Ferra manages.
            let context_ptr = &captured_value as *const Int as *mut c_void; 
                                            
            // register_c_callback(ferra_trampoline_for_c, context_ptr); // Assuming register_c_callback is available
            // IMPORTANT: Lifetime of `captured_value` (and thus `context_ptr`) must outlive
            // any potential calls to the callback by the C library.
            // If C library unregisters callback or context is no longer needed, Ferra might need to free context_ptr if it was heap allocated by Ferra.
        }
        ```

*   **Lifetime and Safety for `userdata`**:
    *   If `userdata` points to Ferra-managed memory (e.g., a `Box`ed closure context), Ferra is responsible for ensuring this memory remains valid for as long as the C library might use the callback.
    *   If the C library copies the `userdata` or has its own lifecycle for it, that must be understood.
    *   When the callback is unregistered or no longer needed, Ferra might need to explicitly free the `userdata` memory if it was heap-allocated by Ferra (e.g., by `Box::from_raw` in a corresponding unregister/cleanup function). This addresses TBD FFI-C-5.

*   **Panic Handling**: As emphasized, panics within Ferra callback logic (including within the trampoline) **must be caught** and translated into a C-compatible error return (e.g., a specific integer value). They must not unwind into the C caller.

This approach allows Ferra to integrate with C APIs that use callbacks, while managing the complexities of stateful closures and safety across the FFI boundary.

### 4.6. Generating C/C++ Header Files

To facilitate the use of Ferra libraries from C and C++ projects, it's highly beneficial for the Ferra toolchain to be able to automatically generate C/C++ compatible header files (`.h` or `.hpp`). These headers would declare the functions and data types that Ferra exports with a C ABI.

*   **Purpose**:
    *   Provide C/C++ developers with type-safe declarations for calling Ferra functions.
    *   Define C struct layouts corresponding to Ferra `data` classes marked `#[repr(C)]`.
    *   Make it easier to integrate Ferra-compiled static or dynamic libraries into C/C++ build systems.
    *   Reduce manual effort and potential for errors in writing FFI declarations on the C/C++ side.

*   **Tooling Support**:
    *   This functionality could be provided by:
        *   A dedicated subcommand of the `lang` CLI tool (e.g., `lang build --emit-header <output_path>` or `lang generate-header --crate <crate_name> -o <output_file>`).
        *   A separate utility tool (e.g., `ferra-bindgen` or `ferra-cheader`).
    *   The tool would analyze a compiled Ferra library (or its source code/AST metadata) to identify all `pub extern "C"` functions and `pub #[repr(C)]` data types.

*   **Build Script Integration (Triggering Header Generation)**:
    *   The Ferra build system (via `build.ferra` scripts, see Section 7) could provide directives or hooks to trigger header generation automatically as part of the build process.
    *   For example, a build script might specify that a header file should be generated if the Ferra library's public FFI API changes, similar to how `cargo:rerun-if-changed=` works in Rust's Cargo for build scripts.
    *   This would ensure that C/C++ header files are kept up-to-date with the Ferra library's exported interface.

*   **Content of Generated Headers**:
    *   **Include Guards**: Standard `#ifndef LIB_NAME_H ... #define LIB_NAME_H ... #endif` to prevent multiple inclusion.
    *   **`extern "C"` Guards for C++**: Wrap declarations in `#ifdef __cplusplus extern "C" { #endif ... #ifdef __cplusplus } #endif` to ensure C linkage when included in C++ source files.
    *   **Function Declarations**: For each `pub extern "C"` Ferra function, generate a corresponding C function prototype.
        *   Ferra function names (after `#[no_mangle]` or `#[export_name]`) become C function names.
        *   Ferra FFI-safe parameter and return types are mapped to their C equivalents (as detailed in Section 4.2).
    *   **Struct Definitions**: For each `pub #[repr(C)]` Ferra `data` class, generate an equivalent C `struct` definition.
        *   Field names and types must match the C-compatible layout.
        *   Padding and alignment should be consistent with what the Ferra compiler produces for `#[repr(C)]`.
    *   **Enum Definitions (C-Style)**: For Ferra enums marked for C representation (e.g., `#[repr(c_int)]`), generate C `enum` definitions or a series of `#define` constants.
    *   **Type Aliases (`typedef`)**:
        *   For opaque handles returned by Ferra (e.g., `*mut c_void` representing a Ferra object), a C `typedef` can provide a named opaque pointer type (e.g., `typedef void* MyFerraHandle;`).
        *   For C-compatible function pointer types that Ferra expects as callbacks, generate `typedef`s.
    *   **Standard C Includes**: May need to include standard C headers like `<stdint.h>` for fixed-width integer types (e.g., `int32_t`) or `<stdbool.h>` for `bool`, if these are used in the generated declarations.
    *   **Documentation (Optional)**: Potentially translate Ferra doc comments for exported items into C-style Doxygen comments in the header.

*   **Example of Generated Header**:
    Given the following Ferra code:
    ```ferra
    // In my_ferra_lib

    #[repr(C)]
    pub data Point {
        x: f64,
        y: f64,
    }

    #[no_mangle]
    pub extern "C" fn create_point(x: f64, y: f64) -> *mut Point {
        let p = Box::new(Point { x: x, y: y }); // Conceptual boxing
        return Box::into_raw(p);
    }

    #[no_mangle]
    pub extern "C" fn get_x(p: *const Point) -> f64 {
        unsafe { return (*p).x; }
    }

    #[no_mangle]
    pub extern "C" fn free_point(p: *mut Point) {
        if !p.is_null() {
            unsafe { let _ = Box::from_raw(p); } // Deallocate
        }
    }

    // Callback type
    pub type CNotificationCallback = extern "C" fn(message: *const c_char, user_data: *mut c_void);

    #[no_mangle]
    pub extern "C" fn register_notifier(cb: CNotificationCallback, data: *mut c_void) {
        // ... store cb and data ...
    }
    ```
    The generated `my_ferra_lib.h` might look like:
    ```c
    #ifndef MY_FERRA_LIB_H
    #define MY_FERRA_LIB_H

    #include <stdint.h> // For types like int32_t, double might be standard
    #include <stddef.h> // For size_t (if used, though not in this direct example)

    #ifdef __cplusplus
    extern "C" {
    #endif

    // Forward declaration for opaque types if not fully defined,
    // or full definition if #[repr(C)]
    typedef struct Point Point;

    // Actual struct definition if fields are public and layout is C-compatible
    struct Point {
        double x;
        double y;
    };

    Point* create_point(double x, double y);
    double get_x(const Point* p);
    void free_point(Point* p);

    typedef void (*CNotificationCallback)(const char* message, void* user_data);
    void register_notifier(CNotificationCallback cb, void* data);

    #ifdef __cplusplus
    } // extern "C"
    #endif

    #endif // MY_FERRA_LIB_H
    ```

*   **Type Mapping Consistency**: The tool must ensure that the C types generated in the header precisely match the ABI representation used by the Ferra compiler for its FFI-safe types when `extern "C"` is specified.
*   **Build Integration**: The header generation tool should be easily integrable into both Ferra's build process (for library authors) and C/C++ build systems (for library consumers).

This feature significantly enhances the usability of Ferra libraries from C and C++, providing a type-safe and convenient way to interface with them.

## 5. Detailed Data Marshalling

This section provides a more in-depth look at how different categories of data types are marshalled (i.e., converted and managed) across the FFI boundary between Ferra and C. The goal is to ensure data is correctly interpreted and that memory safety is maintained according to each language's rules.

### 5.1. Primitives (Int, Float, Bool, Char)

Primitive data types are generally the most straightforward to marshall across the FFI boundary because their memory representations are often directly compatible or have well-defined C ABI equivalents.

*   **Integer Types**:
    *   **Ferra to C**:
        *   Ferra's fixed-width integer types (`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`) are passed by value and are directly ABI-compatible with their corresponding C standard integer types (e.g., `int8_t`, `uint32_t`, `long long` etc., or via type aliases like `c_int`).
        *   Ferra's general `Int` (defaulting to 64-bit) would be passed as a 64-bit integer (e.g., `int64_t` or `long long` in C).
        *   No special marshalling beyond ensuring type size and signedness match the C function's expectation is typically needed.
    *   **C to Ferra**:
        *   C integer types passed to an `extern "C"` Ferra function are received by value.
        *   The Ferra function signature would use the corresponding Ferra fixed-width integer type or an appropriate `c_` type alias.
    *   **Representation**: These are passed directly on the stack or in registers as per the C calling convention. No heap allocation or pointers are involved for the primitive value itself.

*   **Floating-Point Types**:
    *   **Ferra to C**:
        *   Ferra `f32` is passed by value and is ABI-compatible with C `float`.
        *   Ferra `f64` is passed by value and is ABI-compatible with C `double`.
    *   **C to Ferra**:
        *   C `float` and `double` are received by value as Ferra `f32` and `f64` respectively.
    *   **Representation**: Passed directly on the stack or in floating-point registers as per the C calling convention.

*   **Boolean Types**:
    *   **Ferra to C**:
        *   Ferra `Bool` (values `true`, `false`) is marshalled to a C `bool` (from `<stdbool.h>`, typically an `_Bool`) or a C integer type (like `int` or `char`) as defined by the target C ABI for boolean representation. Often, this is 0 for false and 1 for true.
        *   Ferra will ensure its `true`/`false` values are converted to the C ABI's expected integer representation for booleans.
    *   **C to Ferra**:
        *   A C `bool` or integer representing a boolean (where 0 is false, non-zero is true, as per C convention) is converted to Ferra `Bool`.
        *   The Ferra `extern "C"` function signature would declare the parameter as `Bool` (or a `c_bool` alias).
    *   **Representation**: Usually passed as a small integer type (e.g., a byte or an int) by value.

*   **Character Types**:
    *   **Ferra `Char` (Unicode Scalar Value) to C**:
        *   Ferra's `Char` represents a Unicode scalar value (up to 21 bits, typically stored in 32 bits).
        *   When passing to a C function expecting a simple `char` (typically for ASCII or a single byte of a multi-byte encoding), the Ferra `Char` must be appropriately converted.
            *   If the C `char` is intended for ASCII, and the Ferra `Char` is outside the ASCII range, this is a lossy conversion or an error. The FFI call should likely ensure the `Char` is ASCII, or the C function must be ableto handle wider character types.
            *   It's often safer for C APIs to accept `uint32_t` or `wchar_t` (if its size is appropriate for Unicode scalars on the platform) if they expect a full Unicode character.
        *   A Ferra `Char` can be passed as a `c_int` or `uint32_t` to C if the C function expects a raw Unicode code point.
    *   **C `char` to Ferra `Char`**:
        *   If a C function passes a `char` (e.g., `signed char` or `unsigned char`) to Ferra:
            *   If it's intended as a byte value, Ferra would receive it as an `i8` or `u8` (or `c_char`).
            *   If it's intended as a character and needs to become a Ferra `Char`, it's assumed to be an ASCII character or part of a UTF-8 sequence that needs further processing. For a single `c_char` parameter to become a Ferra `Char`, it typically implies ASCII.
    *   **`wchar_t` (C)**:
        *   The size of `wchar_t` is platform-dependent (e.g., 16 bits on Windows, 32 bits on Linux).
        *   If C uses `wchar_t` for Unicode, Ferra needs a corresponding `c_wchar` type alias. Marshalling would involve conversion to/from Ferra `Char` or `String` depending on whether it's a single character or a string. This is more complex than simple `char` and depends on the encoding `wchar_t` represents (e.g., UTF-16 on Windows).
    *   **Representation**: Passed by value as an integer type. For `Char`, this is usually the integer value of the Unicode code point.

*   **General Marshalling Notes for Primitives**:
    *   **No Ownership Transfer**: Primitive types are passed by value (copied). There are no ownership or lifetime concerns for the primitive values themselves.
    *   **Direct Compatibility**: The primary concern is ensuring the size and interpretation (signed/unsigned for integers, float precision) match between the Ferra declaration and the C function's actual signature. Using the FFI type aliases (e.g., `c_int`, `c_double`) is crucial for this.
    *   **No `unsafe` Needed for Passing**: The act of passing primitive values themselves usually doesn't require an `unsafe` block beyond the `unsafe` block needed for the FFI call itself, as their representations are directly compatible.

### 5.2. Pointers (Raw Pointers `*T`, `*mut T`, `*const T`)

Raw pointers are a fundamental part of C interoperability. Ferra provides `*const T` (equivalent to C's `const T*`) and `*mut T` (equivalent to C's `T*`) for FFI. These are "unsafe" pointers, meaning the Ferra compiler does not track their lifetimes or guarantee what they point to is valid, unlike Ferra's safe references (`&T`, `&mut T`).

*   **Representation**:
    *   Ferra `*const T` and `*mut T` are represented as raw memory addresses, directly compatible with C pointers.
    *   `T` must be an FFI-safe type, meaning its Ferra representation is compatible with its intended C representation (e.g., primitives, `#[repr(C)]` structs).
    *   A special type, `c_void` (likely an empty enum or struct in Ferra, aliased in `core::ffi`), can be used with raw pointers (`*const c_void`, `*mut c_void`) to represent C's `void*`.

*   **Ferra to C**:
    *   **Creating Raw Pointers in Ferra**:
        *   From Ferra references: `let my_val: Int = 10; let c_ptr: *const Int = &my_val;` (Implicit conversion from `&T` to `*const T`, and `&mut T` to `*mut T`).
        *   From `Box<T>` (if Ferra has an equivalent for heap allocation): `Box::into_raw(my_boxed_data)` would yield a `*mut T`.
        *   From integer addresses (highly unsafe, requires `unsafe` block): `let addr: usize = 0x12345; let c_ptr = addr as *const u8;`
    *   **Passing to C**: Ferra `*const T` or `*mut T` can be passed directly to C functions expecting `const CEquivOfT*` or `CEquivOfT*`.
    *   **Lifetime**: When passing a raw pointer derived from a Ferra reference or Ferra-managed allocation (like `Box`), the Ferra developer is responsible for ensuring the underlying Ferra data outlives the C code's use of the pointer. The Ferra compiler does not track this across the FFI boundary.
    *   **Null Pointers**: Ferra can create and pass null pointers using a construct like `core::ptr::null()` or `core::ptr::null_mut()`, or by casting `0` to a pointer type in an `unsafe` block.

*   **C to Ferra**:
    *   **Receiving Pointers from C**: C functions can return pointers, which Ferra receives as `*const T` or `*mut T`.
        ```ferra
        extern "C" {
            fn c_get_data_readonly() -> *const c_int;
            fn c_get_data_mutable() -> *mut c_int;
            fn c_get_opaque_handle() -> *mut c_void;
        }
        ```
    *   **Dereferencing (Unsafe)**: Reading from or writing to memory via a raw pointer received from C is an `unsafe` operation in Ferra because the compiler cannot guarantee the pointer is valid (non-null, points to allocated and live memory of the correct type, correctly aligned).
        ```ferra
        fn process_c_data() unsafe {
            let data_ptr = c_get_data_readonly();
            if !data_ptr.is_null() { // Always check for null!
                let value = *data_ptr; // Unsafe dereference
                println("Value from C: " + String::from_int(value));
            }

            let mutable_ptr = c_get_data_mutable();
            if !mutable_ptr.is_null() {
                *mutable_ptr = 42; // Unsafe write
            }
        }
        ```
    *   **Null Pointer Checks**: Ferra code **must** check pointers received from C for null before dereferencing, unless the C API explicitly guarantees a non-null return. Helper methods like `pointer.is_null()` should be available.
    *   **Memory Ownership**:
        *   If C returns a pointer to memory it allocated, Ferra is usually responsible for calling a C deallocation function for that pointer when done (see Section 3.4 and 5.3+).
        *   If C returns a pointer to static C data or data managed by the C library with its own lifetime, Ferra must respect that lifetime and not attempt to deallocate it.

*   **Pointer Arithmetic**:
    *   Ferra may provide methods on raw pointers for offset calculations (e.g., `pointer.offset(count)`), similar to pointer arithmetic in C or Rust. These operations are `unsafe`.
    *   Care must be taken to ensure pointer arithmetic does not go out of bounds of allocated memory.

*   **Casting Raw Pointers**:
    *   Raw pointers can be cast between different types using `as *const NewType` or `as *mut NewType` within an `unsafe` block. This is highly unsafe if the types are not layout-compatible or if the memory does not actually contain an object of `NewType`.

*   **Function Pointers**:
    *   Marshalling of C function pointers to Ferra function pointer types (and vice-versa for callbacks) is covered in Section 5.7. The pointers themselves are addresses and marshall like other raw pointers.

Marshalling raw pointers mainly involves direct use of Ferra's `*const T` and `*mut T` types. The primary challenge is managing safety, lifetimes, and memory ownership manually, which is why operations involving raw pointers are `unsafe`.

### 5.3. Strings (Ferra `String` <-> C `char*`)

Marshalling strings between Ferra and C is a common requirement and involves careful handling of encoding, null termination, and memory ownership. Ferra `String`s are internally UTF-8 encoded, dynamically sized, and manage their own memory, while C strings are typically null-terminated arrays of `char` (`char*`) with varying encoding expectations (often ASCII or system locale, but UTF-8 is common in modern C APIs).

*   **Ferra `String` to C `const char*` (for C functions that only read)**:
    *   **Goal**: Provide C with a pointer to a null-terminated sequence of bytes, usually UTF-8.
    *   **Mechanism**:
        1.  **Obtaining a Pointer**: Ferra `String` should provide an `unsafe` method (e.g., `as_ptr() -> *const u8` or `as_c_str_ptr() -> *const c_char`) that returns a pointer to its internal UTF-8 byte buffer.
        2.  **Null Termination**: Ferra `String`s are not guaranteed to be null-terminated internally.
            *   If the C API strictly requires a null-terminated string, and the Ferra `String` isn't already, Ferra code must create a temporary, null-terminated copy before passing the pointer. This involves allocating a new buffer (Ferra or C heap), copying the string content, appending a null byte, and then passing the pointer to this temporary buffer. The temporary buffer must be managed correctly (e.g., freed after the C call if heap-allocated).
            *   Alternatively, if the C function accepts a pointer and a length, null termination might not be strictly needed, and Ferra can pass `string.as_ptr()` and `string.byte_len()`. This is often safer and more efficient.
    *   **Lifetime**: The pointer passed to C is only valid as long as the original Ferra `String` is live and its internal buffer has not been reallocated (e.g., due to mutation, though Ferra `String`s are immutable by default, operations like concatenation create new strings). This is a critical lifetime concern for the Ferra developer.
    *   **Encoding**: Assumed to be UTF-8 unless the C API specifies otherwise (requiring transcoding).
    *   **Example**:
        ```ferra
        extern "C" {
            fn c_puts(s: *const c_char); // C puts expects a null-terminated string
            fn c_process_bytes(bytes: *const u8, len: usize);
        }

        fn pass_string_to_c(my_ferra_str: String) unsafe {
            // Scenario 1: C function expects null-terminated string
            // This requires ensuring null termination.
            // Conceptual: stdlib might provide `my_ferra_str.to_c_string()` -> temp CString type that handles null term.
            // Or manually:
            let c_compatible_string = my_ferra_str + "\0"; // Create a new Ferra string with null. (Simplistic)
                                                          // A robust solution would use a dedicated CString type or buffer.
            c_puts(c_compatible_string.as_ptr() as *const c_char);


            // Scenario 2: C function takes pointer and length
            c_process_bytes(my_ferra_str.as_ptr(), my_ferra_str.byte_len());
        }
        ```

*   **C `const char*` to Ferra `String` (for Ferra to use a string from C)**:
    *   **Goal**: Create a Ferra `String` from a C string.
    *   **Mechanism**: This almost always involves **copying** the data from the C string into a new Ferra `String` allocation. Ferra cannot take ownership of arbitrary `char*` memory from C safely.
        1.  **Determine Length**: If the C string is null-terminated, Ferra code must find the null terminator to determine its length (e.g., using a C `strlen` equivalent called via FFI, or by iterating). If the C API provides a length, use that.
        2.  **Validate UTF-8**: If the C string is expected to be UTF-8, the bytes should be validated during or after copying. If invalid UTF-8 is encountered, the conversion should either fail (return `Result`) or perform lossy conversion (e.g., replace invalid sequences with U+FFFD).
        3.  **Allocate and Copy**: Allocate a new Ferra `String` and copy the (UTF-8 validated) byte sequence into it.
    *   **Helper Function**: A Ferra stdlib function like `String::from_c_str(c_ptr: *const c_char) -> Result<String, Utf8Error>` or `String::from_c_bytes(c_ptr: *const u8, len: usize) -> Result<String, Utf8Error>` is essential.
    *   **Lifetime**: The `c_ptr` must be valid for the duration of the copy. Once copied, the Ferra `String` is independent.
    *   **Example**:
        ```ferra
        extern "C" {
            fn get_error_message() -> *const c_char; // Returns a pointer to a static C string or C-managed string
        }

        fn handle_c_error_message() -> Option<String> unsafe {
            let c_msg_ptr = get_error_message();
            if c_msg_ptr.is_null() {
                return None;
            }
            // Conceptual: String::from_c_str copies and validates
            match String::from_c_str(c_msg_ptr) { 
                Ok(ferra_msg) => return Some(ferra_msg),
                Err(_) => {
                    eprintln("Failed to convert C string to Ferra String (invalid UTF-8?)");
                    return None;
                }
            }
        }
        ```

*   **Modifiable Strings (`char*` passed to C for writing, or C `char*` Ferra needs to modify)**:
    *   **Ferra Buffer for C to Write Into**:
        *   Ferra allocates a `Vector<u8>` or similar mutable byte buffer.
        *   Passes `buffer.as_mut_ptr()` and `buffer.capacity()` to the C function.
        *   After C writes to it (and ideally returns the number of bytes written or null-terminates), Ferra can construct a `String` from the valid UTF-8 portion of the buffer.
        *   The Ferra code must handle potential buffer overflows if C writes too much.
    *   **Modifying a C String in Ferra**:
        *   If Ferra receives a `*mut c_char` from C that it's allowed to modify, it must do so respecting C's memory bounds and encoding. This is highly `unsafe`. Usually, it's better to copy to a Ferra `String`, modify, and then copy back if necessary using the pattern above.

*   **Memory Ownership Summary for Strings**:
    *   **Ferra `String` -> C `const char*`**: Ferra retains ownership. C borrows. Pointer validity is Ferra's responsibility during the borrow.
    *   **C `const char*` -> Ferra `String`**: Ferra creates a new owned copy. C retains ownership of its original string.
    *   **Buffers for C to write into**: Ferra owns the buffer passed to C.
    *   **Ferra modifying C `char*`**: C owns the buffer. Ferra operates on borrowed memory.

*   **`core::ffi::CString` and `core::ffi::CStr` (Conceptual, Rust-inspired)**:
    *   To manage null termination and ownership guarantees more robustly, Ferra might introduce helper types in its FFI module:
        *   `CString`: An owned, null-terminated byte sequence suitable for passing to C functions that require ownership or a modifiable buffer they don't own. Created from a Ferra `String`.
        *   `CStr`: A borrowed, null-terminated byte slice, analogous to `&str` but for C strings. Created by wrapping a `*const c_char` that is known to be null-terminated and valid. Provides safe operations (like length, conversion to Ferra `&str` if UTF-8).
    *   These types would encapsulate much of the manual pointer and null-termination logic. For v0.1, basic pointer passing with manual management might be the starting point, with these helpers as a highly desirable addition. (Addresses TBD FFI-C-3). Their full API and implementation details would be specified as part of the `core::ffi` module design.

Handling strings correctly is one of the most common and error-prone parts of FFI. Clear patterns and helper utilities are essential.

### 5.4. Structs/Data Classes (`#[repr(C)]` equivalent for layout control)

Marshalling structs (Ferra `data` classes) across the FFI boundary requires ensuring that their memory layout is compatible between Ferra and C. This is typically achieved using an attribute similar to Rust's `#[repr(C)]`.

*   **`#[repr(C)]` Attribute (Conceptual)**:
    *   When a Ferra `data` class is intended for FFI with C, it **must** be annotated with `#[repr(C)]` (or an equivalent Ferra attribute, TBD FFI-C-2). The precise syntax for this attribute and its variants (e.g., for alignment, packing) must be defined in `SYNTAX_GRAMMAR_V0.1.md` as part of resolving FFI-C-2.
    *   This attribute instructs the Ferra compiler to lay out the fields of the `data` class in memory in a way that is compatible with how a C compiler would lay out an equivalent C `struct`. This includes:
        *   **Field Order**: Fields are laid out in the order they are declared.
        *   **Padding and Alignment**: Standard C padding and alignment rules for the target platform are applied. This ensures that C code can correctly access fields by offset.
    *   Without `#[repr(C)]`, the Ferra compiler is free to reorder fields or use different padding/alignment for optimization, which would make the layout incompatible with C.
    *   All fields within a `#[repr(C)]` data class must themselves be FFI-safe types (primitives, raw pointers, or other `#[repr(C)]` data classes).

*   **Ferra `data` Class to C `struct`**:
    *   **Passing by Value**:
        *   If a `#[repr(C)]` Ferra `data` class is passed by value to a C function, its bit pattern is copied onto the stack or into registers according to the C calling convention for structs.
        *   The C function would declare a parameter of the equivalent C `struct` type.
        *   Ownership of the data itself (if it contains heap-allocated Ferra types like `String` not represented as raw C pointers) is complex and usually avoided for by-value FFI structs; typically, `#[repr(C)]` structs for FFI contain only C-compatible primitive types or raw pointers.
    *   **Passing by Pointer (`*const MyReprCData`, `*mut MyReprCData`)**:
        *   A pointer to a Ferra `#[repr(C)]` data instance can be passed to C. C receives a pointer to a C-compatible struct.
        *   Ferra usually retains ownership of the underlying data. The C code borrows it.
        *   The lifetime of the Ferra data must exceed the C code's use of the pointer.
        *   Dereferencing the pointer in C to access fields is standard C struct field access.
        ```ferra
        #[repr(C)]
        pub data CCoords {
            x: f64,
            y: f64,
            id: u32,
        }

        extern "C" {
            fn process_coords_val(c: CCoords);
            fn process_coords_ptr(cp: *const CCoords);
            fn modify_coords_ptr(cp: *mut CCoords);
        }

        fn call_c_with_coords() unsafe {
            let my_coords = CCoords { x: 1.0, y: 2.0, id: 101 };
            process_coords_val(my_coords); // Passed by value (copied)

            process_coords_ptr(&my_coords); // Passed by const pointer (borrowed)
            
            var mutable_coords = CCoords { x: 3.0, y: 4.0, id: 102 };
            modify_coords_ptr(&mut mutable_coords); // Passed by mutable pointer (mutably borrowed)
        }
        ```

*   **C `struct` to Ferra `data` Class**:
    *   **Receiving by Value**: If a C function returns a C `struct` by value, a Ferra function declared to receive a `#[repr(C)]` data class of the equivalent layout will receive a copy.
    *   **Receiving by Pointer**: If a C function returns a pointer to a C `struct`, Ferra receives it as `*const MyReprCData` or `*mut MyReprCData`.
        *   Ferra code must `unsafe`ly dereference this pointer to access fields.
        *   **Memory Ownership**: Critical! If C allocated the struct, Ferra must know how and when (if ever) to tell C to free it. If the pointer is to memory C expects to remain valid (e.g., static or C-library-managed), Ferra must not try to free it. This usually requires C to also provide a deallocation function.
        ```ferra
        extern "C" {
            fn get_default_coords() -> CCoords; // Returns by value
            fn allocate_coords_on_c_heap(x:f64, y:f64, id:u32) -> *mut CCoords;
            fn free_c_allocated_coords(ptr: *mut CCoords);
        }

        fn use_c_coords() unsafe {
            let coords_val = get_default_coords();
            println("C val: " + String::from_float(coords_val.x));

            let coords_ptr = allocate_coords_on_c_heap(5.0, 6.0, 201);
            if !coords_ptr.is_null() {
                println("C ptr x: " + String::from_float((*coords_ptr).x));
                (*coords_ptr).id = 202; // Modify if mutable
                free_c_allocated_coords(coords_ptr); // Ferra calls C's free function
            }
        }
        ```

*   **Nested Structs**:
    *   If a `#[repr(C)]` Ferra `data` class contains fields that are themselves other `#[repr(C)]` data classes, the layout compatibility extends hierarchically.

*   **Unions (C `union`)**:
    *   Ferra's direct equivalent for C unions is TBD. For v0.1, if interop with a C union is needed, it might be represented in Ferra as a `#[repr(C)]` struct with a size matching the union and fields for accessing different interpretations (highly `unsafe`), or by only exposing one variant of the union's interpretation via C wrapper functions. True tagged unions in Ferra (like `enum` with data) are not directly C `union`s.

*   **Bitfields (C `struct` bitfields)**:
    *   Direct mapping of C bitfields to Ferra `data` class fields is complex and often non-portable due to C compiler-specific layout.
    *   For v0.1, Ferra will likely not have direct support for defining or matching C bitfield layouts. Interop with C structs containing bitfields usually requires C-side helper functions to get/set bitfield values via regular integer types.

The `#[repr(C)]` attribute (or its Ferra equivalent) is the cornerstone for struct/data class marshalling, ensuring predictable memory layout across the FFI boundary.* 

### 5.5. Arrays and Vectors/Slices

Marshalling arrays, Ferra `Vector<T>`, and slices (`&[T]`, `&mut [T]`) involves passing a pointer to a contiguous block of memory and usually its length.

*   **C Arrays (Fixed-size and Pointers to elements)**:
    *   **Fixed-size C arrays in structs**: If a C struct contains a fixed-size array (e.g., `int arr[10];`), a `#[repr(C)]` Ferra `data` class can map this directly using Ferra's fixed-size array type (once Ferra has one, for v0.1 this might be `[CType; 10]` if supported or a tuple of CTypes).
        ```ferra
        // C: struct MyData { int values[5]; };
        // Ferra (conceptual, assuming fixed-size array syntax `[Type; Size]`)
        #[repr(C)]
        data MyCData {
            values: [c_int; 5], // Requires Ferra to support C-compatible fixed-size arrays
        }
        ```
        For v0.1, if fixed-size arrays are not directly supported in Ferra `data` types with `#[repr(C)]`, interop might require treating them as opaque blobs or accessing via pointers and manual offsets, or C-side helpers.
    *   **Pointers to C arrays (`T*` used as array)**: C functions often take a pointer to the first element of an array and a separate length parameter.
        *   **Ferra to C**: Ferra can pass a `*const T` or `*mut T` (obtained from a Ferra `Vector`, slice, or other contiguous memory) along with a `c_size_t` length.
        *   **C to Ferra**: A C function might return a pointer to an array it allocated (requiring Ferra to know its length and how to free it) or fill a buffer provided by Ferra.

*   **Ferra `Vector<T>` and Slices (`&[T]`, `&mut [T]`) to C**:
    *   **Representation**: These are typically passed to C as a two-component "slice" or "fat pointer":
        1.  A raw pointer to the first element (`*const ElementCType` or `*mut ElementCType`).
        2.  The number of elements (length) as `c_size_t`.
    *   **Ferra `Vector<T>`**:
        *   To pass a `Vector<T>` to C, Ferra provides `vector.as_ptr()` (or `vector.as_mut_ptr()`) and `vector.len()`.
        *   Ferra retains ownership of the `Vector`'s memory. C receives a borrow.
        *   The C function must not access elements beyond the provided length.
        *   The pointer is valid as long as the `Vector` is live and its internal buffer isn't reallocated (e.g., by `push` if it causes growth).
    *   **Ferra Slices (`&[T]`, `&mut [T]`)**:
        *   These already represent a pointer and length internally in Ferra. They can be directly converted to `(*const ElementCType, c_size_t)` or `(*mut ElementCType, c_size_t)` for C.
        *   The lifetime of the slice borrow in Ferra must encompass the duration of the C call.
    *   **Element Type `T`**: Must be FFI-safe. If `T` is a `data` class, it must be `#[repr(C)]`.
    *   **Example (Passing a slice of Ferra data to C)**:
        ```ferra
        #[repr(C)]
        pub data MyItem { id: i32, value: f64 }

        extern "C" {
            fn process_items(items_ptr: *const MyItem, count: usize);
        }

        fn send_items_to_c(items_vec: Vector<MyItem>) unsafe {
            // Using a slice from the vector
            let item_slice: &[MyItem] = items_vec.as_slice(); // Conceptual method
            process_items(item_slice.as_ptr(), item_slice.len());
        }

        fn send_fixed_array_to_c() unsafe {
            let items_array = [ // Assuming Ferra supports array literals that can become slices
                MyItem { id: 1, value: 1.0 },
                MyItem { id: 2, value: 2.0 },
            ];
            let item_slice: &[MyItem] = &items_array; // Take a slice
            process_items(item_slice.as_ptr(), item_slice.len());
        }
        ```

*   **C Array Data to Ferra `Vector<T>` or Slice**:
    *   If a C function provides a pointer to an array and its length, Ferra can:
        1.  **Create a Ferra Slice (Borrowing from C)**: `unsafe { core::slice::from_raw_parts(c_ptr, c_len) }` (Rust-like conceptual API). This creates a Ferra slice (`&[ElementType]` or `&mut [ElementType]`) that borrows directly from the C-managed memory.
            *   This is highly `unsafe` because Ferra cannot guarantee the lifetime or validity of the C memory. The Ferra slice must not outlive the C data.
            *   Useful for temporary, read-only access or careful in-place modification.
        2.  **Copy to a Ferra `Vector<T>`**: Iterate over the C array (from pointer and length) and copy each element into a new Ferra `Vector<T>`. This is safer as Ferra then owns the data.
            *   Requires conversion for each element if `ElementType` in Ferra is different from the C element type (e.g., C `char*` elements to Ferra `String` elements).

*   **Memory Ownership**:
    *   **Ferra `Vector`/slice -> C `(pointer, length)`**: Ferra owns the data; C borrows.
    *   **C `(pointer, length)` -> Ferra slice**: C owns the data; Ferra borrows (unsafe).
    *   **C `(pointer, length)` -> Ferra `Vector`**: Ferra makes an owned copy. C retains ownership of its original array.
    *   If a C function returns a pointer to an array it dynamically allocated, Ferra must call the appropriate C deallocation function, and also know the array's size (either returned or by convention).

Marshalling arrays and vectors/slices centers on the `(pointer, length)` pattern and careful management of lifetimes and ownership, especially when Ferra borrows C-owned array data.

### 5.6. Enums (Representation in C)

Marshalling enums between Ferra and C requires ensuring that their underlying integer representation and interpretation are compatible.

*   **C-Style Enums in Ferra**:
    *   Ferra needs a way to define enums that are layout-compatible with C enums. C enums are fundamentally integer types, where each enumerator has an associated integer value.
    *   **`#[repr(IntType)]` Attribute**: A Ferra enum intended for FFI with C should be annotated with an attribute like `#[repr(C_Enum)]` or, more generally, `#[repr(IntType)]` (e.g., `#[repr(c_int)]`, `#[repr(u8)]`) to specify the exact underlying integer type that C expects. (This is TBD FFI-C-2, and its syntax must be added to `SYNTAX_GRAMMAR_V0.1.md`).
        *   If no `#[repr(IntType)]` is specified, a default underlying integer type (e.g., `c_int`) might be assumed for `extern "C"` enum contexts, but explicit representation is safer for FFI.
    *   The Ferra enum variant values would correspond to the C enumerator values.
    *   **Example**:
        ```ferra
        // C code:
        // typedef enum {
        //     STATUS_OK = 0,
        //     STATUS_WARN = 1,
        //     STATUS_ERROR = 2
        // } MyStatus;
        // int process_status(MyStatus s);

        // Ferra code:
        #[repr(c_int)] // Or the appropriate integer type for MyStatus in C
        pub data MyStatus { // Assuming Ferra uses `data` for C-like enums, or a dedicated `enum` syntax
            OK = 0,
            WARN = 1,
            ERROR = 2,
        }
        // If Ferra has a distinct `enum` syntax for this:
        // #[repr(c_int)]
        // pub enum MyStatus { OK = 0, WARN = 1, ERROR = 2 }


        extern "C" {
            fn process_status(s: MyStatus) -> c_int;
        }

        fn call_c_with_status() unsafe {
            let status_ok = MyStatus::OK; // Or `MyStatus { OK }` depending on Ferra enum syntax
            process_status(status_ok);

            let status_err = MyStatus::ERROR;
            process_status(status_err);
        }
        ```

*   **Marshalling**:
    *   **Ferra to C**: When a Ferra enum value (with `#[repr(IntType)]`) is passed to C, its underlying integer value is passed. This is a direct value copy, similar to other primitive integer types.
    *   **C to Ferra**: When a C function returns an integer value that represents an enum, Ferra receives that integer.
        *   To convert this integer back to the Ferra enum type, Ferra might require an `unsafe` cast or a "transmute"-like operation if the integer value is not guaranteed to match one of the defined enum variants.
        *   Alternatively, a helper function (e.g., `MyStatus::from_c_int(val: c_int) -> Option<MyStatus>`) can be defined in Ferra to safely convert an integer to an enum variant, returning `None` or a default/error variant if the integer doesn't match a known enumerator.

*   **Ferra's Algebraic Data Types (ADTs / Tagged Unions)**:
    *   Ferra's more complex `data` types that represent tagged unions (e.g., `data Result<T,E> { Ok(T), Err(E) }`) are **not** directly equivalent to simple C enums.
    *   Marshalling such Ferra ADTs to C requires defining a C-compatible struct layout that includes a tag field (discriminant) and a C `union` for the payload, along with helper functions. This is a more advanced struct/union marshalling scenario (covered partly in 5.4 and future considerations for complex ADTs). It's not a simple enum marshalling.

*   **Bitfield Enums / Flags**:
    *   If C uses enums as bitflags (where multiple enumerators can be OR'd together), Ferra would typically represent these as integer types (e.g., `u32`) with associated constants for the flags, rather than a direct enum type, for easier bitwise manipulation.
        ```ferra
        // C:
        // enum CFlags { FLAG_A = 1, FLAG_B = 2, FLAG_C = 4 };
        // void set_flags(int c_flags);

        // Ferra:
        const FLAG_A: u32 = 1;
        const FLAG_B: u32 = 2;
        const FLAG_C: u32 = 4;
        type CFlags = u32; // Or c_uint

        extern "C" {
            fn set_c_flags(flags: CFlags);
        }

        fn use_flags() unsafe {
            set_c_flags(FLAG_A | FLAG_C);
        }
        ```

For simple C-style enums, the key is the `#[repr(IntType)]` attribute to ensure the underlying integer values are compatible across the FFI boundary.

### 5.7. Function Pointers

Function pointers are used in C for callbacks, strategy patterns, and other forms of indirect invocation. Ferra needs to be able to both receive C function pointers and pass its own functions (or suitable wrappers) as C-compatible function pointers.

*   **Representation**:
    *   A C function pointer is essentially a memory address of executable code.
    *   In Ferra, a C-compatible function pointer type is declared using `extern "C" fn(ArgTypes...) -> ReturnType`. This type represents a raw function pointer adhering to the C calling convention.
    *   Example (as seen in 3.2):
        ```ferra
        // C: typedef void (*event_handler_t)(int event_code, void* data);
        // Ferra:
        type CEventHandler = extern "C" fn(c_int, *mut c_void);
        ```

*   **Ferra to C (Passing a Ferra function as a C function pointer)**:
    *   **Directly Passable Ferra Functions**: A top-level Ferra function that is already declared as `pub extern "C"` and has an FFI-safe signature can be directly used where a C function pointer of a compatible type is expected. Its name effectively acts as the pointer.
        ```ferra
        // Ferra function suitable as a callback
        #[no_mangle]
        pub extern "C" fn my_ferra_event_handler(event_code: c_int, user_data: *mut c_void) {
            // ... handle event ...
        }

        // C function that takes a callback
        extern "C" {
            fn register_event_handler(handler: CEventHandler, data: *mut c_void);
        }

        fn setup_ferra_callback() unsafe {
            let context: *mut c_void = /* ... some context ... */;
            register_event_handler(my_ferra_event_handler, context);
        }
        ```
    *   **Closures**: As detailed in Section 4.5 ("Callbacks"), passing Ferra closures that capture an environment requires a "trampoline" pattern. The C function receives a pointer to the static `extern "C"` trampoline function, and the closure's context is passed separately as `void* userdata`.
    *   **Type Safety**: The Ferra type `extern "C" fn(...)` ensures type checking on the Ferra side for the signature of the function being passed.

*   **C to Ferra (Receiving a C function pointer in Ferra)**:
    *   **Declaration**: A C function pointer parameter or return type is declared in Ferra using the `extern "C" fn(...)` type.
        ```ferra
        // C function that returns a function pointer or takes one
        extern "C" {
            fn get_processing_function() -> extern "C" fn(c_int) -> c_int;
            fn execute_custom_task(task: extern "C" fn(*mut c_void), data: *mut c_void);
        }
        ```
    *   **Calling the C Function Pointer from Ferra**:
        *   Once Ferra has a value of an `extern "C" fn(...)` type (obtained from C), it can be called like a regular Ferra function.
        *   This call must occur within an `unsafe` block because Ferra cannot guarantee the validity of the function pointer (it might be null, or point to incorrect code) or the safety of executing that external code.
        *   **Null Check**: Always check if a function pointer received from C is null before attempting to call it.
        ```ferra
        fn use_c_function_pointer() unsafe {
            let processor_fn_ptr = get_processing_function();
            if !processor_fn_ptr.is_null() { // Conceptual null check for function pointers
                let result = processor_fn_ptr(10); // Unsafe call
                println("Result from C function pointer: " + String::from_int(result));
            } else {
                eprintln("Received a null function pointer from C!");
            }
        }
        ```
    *   **Memory and Lifetime**: Function pointers themselves are just addresses. There are no complex memory ownership issues for the pointer value itself. However, if the function pointer is associated with context data (like C++ member function pointers or closures passed via `userdata`), the lifetime of that context is critical.

*   **`core::ptr::null_fn()` (Conceptual)**:
    *   Ferra should provide a way to represent a null function pointer, perhaps `core::ptr::null_fn<MyFnType>()` or by allowing `0 as MyFnType` in an `unsafe` context, for initializing or checking function pointers.

Marshalling function pointers primarily involves ensuring type signature compatibility and handling the `unsafe` nature of calling arbitrary code addresses received from C. The trampoline pattern is key for enabling stateful Ferra closures to be used as C callbacks.

### 5.8. Opaque Types / Handles

Opaque types (often called "handles" or "opaque pointers") are a common pattern in C APIs, especially for object-oriented-like libraries or when the internal structure of a type should not be exposed to the C consumer or is not C-compatible. Ferra FFI must support both consuming and producing such opaque types.

*   **Concept**:
    *   An opaque type in C is typically a pointer to an incomplete struct type (`struct MyOpaqueType*`) or a `void*`, where the C client code does not know the internal layout of `MyOpaqueType`.
    *   All operations on the opaque type are performed by calling functions from the library that defined it.

*   **Consuming Opaque C Types in Ferra**:
    *   **Representation in Ferra**:
        *   If C API uses `void*` for an opaque handle: Ferra uses `*mut c_void` or `*const c_void`.
        *   If C API uses `struct MyThing*` where `MyThing` is an incomplete type (only declared, not defined in headers visible to Ferra's FFI bindings): Ferra can represent this by declaring an empty `data` class (or `struct`) and using a pointer to it, or by using `*mut c_void`.
            ```ferra
            // C Header might have:
            // typedef struct GLFWwindow GLFWwindow; // Incomplete type
            // GLFWwindow* glfwCreateWindow(...);
            // void glfwDestroyWindow(GLFWwindow* window);

            // Ferra FFI declaration:
            // Option 1: Using an empty data class for type safety
            #[repr(C)] // May not be strictly needed if it's always behind a pointer
            pub data GLFWwindow; // Opaque, fields not defined in Ferra

            extern "C" {
                fn glfwCreateWindow(width: c_int, height: c_int, title: *const c_char, monitor: *mut c_void, share: *mut c_void) -> *mut GLFWwindow;
                fn glfwDestroyWindow(window: *mut GLFWwindow);
                // ... other GLFW functions taking GLFWwindow* ...
            }

            // Option 2: Using *mut c_void (less type-safe on Ferra side)
            // extern "C" {
            //     fn glfwCreateWindow(...) -> *mut c_void;
            //     fn glfwDestroyWindow(window: *mut c_void);
            // }
            ```
            Using a named empty `data` type (`GLFWwindow` in the example) is generally preferred over raw `*mut c_void` as it provides better type safety on the Ferra side when passing these handles around.
    *   **Usage**: Ferra receives these opaque pointers from C functions and passes them back to other C functions from the same library. Ferra code should not attempt to dereference or inspect the internal structure of these opaque pointers.
    *   **Memory Management**: If the C library allocates an opaque handle (e.g., `create_handle() -> Handle*`), it must also provide a function to free/destroy it (e.g., `destroy_handle(Handle*)`). Ferra code is responsible for calling this deallocation function when the handle is no longer needed (see Section 3.4). Wrappers implementing Ferra's `Drop` trait (when available) are ideal for managing this.

*   **Exposing Ferra Objects as Opaque Handles to C**:
    *   **Use Case**: When a Ferra object's internal structure is complex, contains Ferra-specific types not easily mappable to C (like closures, `Vector<NonCReprType>`), or when Ferra wants to hide implementation details from C.
    *   **Mechanism**:
        1.  Ferra allocates its object on its own heap (e.g., using `Box::new` conceptually).
        2.  Ferra returns a raw pointer to this heap-allocated object, typically cast to `*mut c_void` or a newtyped pointer struct, to the C caller. This raw pointer is the opaque handle.
        3.  Ferra exports other `extern "C"` functions that take this handle (`*mut c_void`) as an argument. Inside these functions, Ferra `unsafe`ly casts the `*mut c_void` back to the actual Ferra object pointer type and operates on it.
        4.  Ferra **must** export a specific function to destroy/deallocate the Ferra object when C is finished with the handle. This function will take the `*mut c_void` handle, cast it back to the Ferra object pointer, and then properly deallocate it using Ferra's memory management (e.g., `Box::from_raw` to run `Drop` and free memory).
    *   **Example** (revisiting from Section 4.2):
        ```ferra
        // Ferra Internal Object (not #[repr(C)])
        data MyFerraInternalObject {
            name: String,
            data_items: Vector<Int>,
        }

        fn create_internal_object(name_str: String) -> MyFerraInternalObject {
            // ...
            return MyFerraInternalObject { name: name_str, data_items: Vector::new() };
        }

        // FFI Exported Functions
        #[no_mangle]
        pub extern "C" fn my_ferra_object_new(name_c_str: *const c_char) -> *mut c_void {
            unsafe {
                let name_ferra_str = match String::from_c_str(name_c_str) {
                    Ok(s) => s,
                    Err(_) => return core::ptr::null_mut(), // Error creating string
                };
                let internal_obj = create_internal_object(name_ferra_str);
                let boxed_obj = Box::new(internal_obj); // Ferra heap allocation
                return Box::into_raw(boxed_obj) as *mut c_void; // Return raw pointer as opaque handle
            }
        }

        #[no_mangle]
        pub extern "C" fn my_ferra_object_add_item(handle: *mut c_void, item: c_int) {
            if handle.is_null() { return; }
            unsafe {
                let internal_obj_ptr = handle as *mut MyFerraInternalObject;
                // (*internal_obj_ptr).data_items.push(item); // Assuming Vector has push
            }
        }

        #[no_mangle]
        pub extern "C" fn my_ferra_object_get_name_len(handle: *const c_void) -> c_size_t {
            if handle.is_null() { return 0; }
            unsafe {
                let internal_obj_ptr = handle as *const MyFerraInternalObject;
                return (*internal_obj_ptr).name.byte_len(); // Assuming String has byte_len
            }
        }

        #[no_mangle]
        pub extern "C" fn my_ferra_object_free(handle: *mut c_void) {
            if handle.is_null() { return; }
            unsafe {
                let _ = Box::from_raw(handle as *mut MyFerraInternalObject); // Deallocates & drops
            }
        }
        ```
    *   **Lifetime**: C code holding an opaque handle to a Ferra object must use the provided `_free` or `_destroy` function to release it. It must not use the handle after it has been freed. Ferra relies on the C caller to manage the handle's lifecycle correctly by calling the destructor.

Opaque handles are a powerful technique for FFI, allowing complex types to be exchanged and managed across language boundaries by treating them as black boxes on one side and providing a clear API for their creation, operation, and destruction.

## 6. Safety and `unsafe`

Interfacing with C code inherently involves operations that fall outside the scope of what the Ferra compiler can statically verify for memory safety and correctness. Ferra's `unsafe` keyword is the mechanism by which developers acknowledge these limitations and take responsibility for upholding safety invariants at the FFI boundary.

*   **When and Why `unsafe` is Required for FFI**:
    *   **Calling External C Functions**: All calls to functions declared within an `extern "C"` block are inherently `unsafe`. The Ferra compiler cannot analyze the C function's body to ensure it's memory safe, thread-safe, or free of other undefined behavior. Therefore, Ferra requires these calls to be within an `unsafe` block or an `unsafe fn`.
        ```ferra
        extern "C" { fn c_function(data: *const c_void); }
        
        fn my_safe_ferra_function(p: *const c_void) {
            // c_function(p); // ERROR: Call to FFI function is unsafe and must be in an unsafe block.
            unsafe {
                c_function(p); // OK: Developer acknowledges the unsafety.
            }
        }
        ```
    *   **Dereferencing Raw Pointers**: Raw pointers (`*const T`, `*mut T`) received from C or created for C do not have Ferra's compile-time lifetime tracking or null-safety guarantees. Dereferencing them (reading `*ptr` or writing `*ptr = val`) is an `unsafe` operation because the pointer could be null, dangling (pointing to deallocated memory), or pointing to memory of an incompatible type or incorrect alignment.
    *   **Accessing Fields of `#[repr(C)]` Unions (if supported)**: If Ferra supports C-like unions with `#[repr(C)]`, accessing fields of such a union would be `unsafe` because the compiler cannot know which field is currently active/valid.
    *   **Implementing `unsafe` Traits (Future)**: If Ferra introduces `unsafe` traits (e.g., for types that can be sent across threads without compiler verification), implementing these would be an `unsafe` operation. This is less directly FFI but related to overriding compiler safety.
    *   **Calling Ferra `unsafe fn`**: Functions explicitly marked `unsafe fn` in Ferra also require an `unsafe` block to call, signaling they have contracts the caller must uphold.

*   **Best Practices for Writing Safe FFI Wrappers**:
    The primary strategy for managing FFI unsafety in Ferra is to create safe, idiomatic Ferra abstractions (wrappers) around `unsafe` FFI calls.
    *   **Minimize the Scope of `unsafe`**: `unsafe` blocks should be as small and localized as possible, ideally only encompassing the direct FFI call or the minimal set of `unsafe` operations (like pointer dereferences). Avoid large `unsafe` blocks that obscure where the actual unsafety lies.
    *   **Encapsulate Unsafe Operations**: Create safe Ferra functions that internally use `unsafe` blocks to call C functions. These wrapper functions should uphold all necessary invariants and translate C error codes/conventions into Ferra `Result` types or panics where appropriate.
        ```ferra
        // Unsafe C declaration
        extern "C" { fn c_get_value(id: c_int, out_val: *mut c_int) -> c_int; /* 0 on success */ }

        // Safe Ferra wrapper
        pub fn ferra_get_value(id: i32) -> Result<i32, String> { // Using Ferra i32
            let mut c_val: c_int = 0;
            let status = unsafe { 
                c_get_value(id as c_int, &mut c_val) 
            };
            if status == 0 {
                return Ok(c_val as i32);
            } else {
                return Err("C function c_get_value failed with status: ".to_string() + String::from_int(status));
            }
        }
        ```
    *   **Input Validation**: Before passing data to C, validate it on the Ferra side if possible (e.g., check for unexpectedly null pointers if the C function expects non-null, ensure string encodings are correct if critical).
    *   **Output Validation/Conversion**: When data is returned from C (especially pointers), validate it (check for null, check lengths if applicable) before converting it to safe Ferra types. Copy data from C-managed memory into Ferra-managed memory whenever Ferra needs to own it.
    *   **Resource Management**: For C resources that require manual deallocation (e.g., memory from `malloc`, file handles from `fopen`), create Ferra wrapper types that implement Ferra's `Drop` trait (once Ferra has one). The `drop` method would `unsafe`ly call the C deallocation function, ensuring resources are cleaned up when the Ferra wrapper goes out of scope. This is a crucial pattern for safety.
    *   **Thorough Documentation**: Clearly document the safety contract of any `unsafe fn` or FFI wrapper. This includes:
        *   Preconditions for calling the function (e.g., "pointer `p` must not be null and must point to `len` initialized bytes").
        *   Postconditions and invariants the function upholds or expects.
        *   Memory ownership rules for all parameters and return values.
        *   Error handling behavior.
        *   Potential for panics (and if they are caught or can unwind).

*   **Common FFI Pitfalls and How `unsafe` Relates**:
    *   **Dangling Pointers**: Passing a pointer to Ferra data that C stores beyond the lifetime of the Ferra data. `unsafe` is a reminder that Ferra's lifetime checks don't cross into C.
    *   **Null Pointer Dereference**: Dereferencing a pointer from C without checking if it's null. `unsafe` requires the developer to be responsible for this check.
    *   **Buffer Overflows**: C function writes past the end of a buffer provided by Ferra. The `unsafe` call doesn't prevent this; Ferra code must pass correct sizes.
    *   **Mismatched Types/Signatures**: Declaring a C function with an incorrect signature in Ferra. This leads to ABI incompatibility and undefined behavior. `unsafe` doesn't fix this; careful declaration is key.
    *   **Incorrect Memory Deallocation**: Ferra freeing memory allocated by C (or vice-versa), or double-freeing. `unsafe` allows raw pointer manipulation, so correct deallocation logic is the developer's duty.
    *   **Unwinding Panics into C**: Ferra functions called from C must not let panics unwind into C. The `extern "C"` function wrapper should catch panics (conceptually an `unsafe` boundary interaction).
    *   **Data Races**: Calling non-thread-safe C code from concurrent Ferra code without external synchronization. The `unsafe` FFI call cannot enforce thread safety within the C code.

*   **Permissions and Sandboxing for FFI**:
    *   All FFI operations, particularly the loading of external libraries and calls to native symbols, are subject to Ferra's capability-based permission system, as detailed in **SECURITY_MODEL.md** (Sections 3.3 and 4.5).
    *   Packages intending to use FFI must declare appropriate `ffi:load:<library_pattern>` and potentially future `ffi:call:<symbol_pattern>` permissions in their `Ferra.toml` manifest.
    *   Failure to declare necessary `ffi:` permissions will result in compile-time or runtime errors as per the enforcement mechanisms in `SECURITY_MODEL.md`.
    *   Furthermore, the underlying OS-level sandboxing mechanisms (e.g., seccomp-bpf for native ELF targets), also described in `SECURITY_MODEL.md` (Section 4.3), will apply to any system calls made by native code invoked via FFI. This provides an additional layer of runtime protection, even if the FFI permission was broadly granted.

The `unsafe` keyword in Ferra FFI is not a license to write arbitrary dangerous code carelessly. Instead, it's a tool that shifts the responsibility for upholding certain safety invariants from the compiler to the programmer for specific, well-understood interactions with external systems like C libraries. Prudent use of `unsafe`, combined with robust wrapper patterns, is key to building safe and reliable systems with FFI.

## 7. Build System and Linker Integration

To successfully use FFI, the Ferra build system must be aware of the external C/C++ libraries that need to be linked, and similarly, C/C++ build systems need to know how to link against Ferra-compiled libraries. Ferra will use a manifest file (e.g., `Ferra.toml` or similar, as per `PACKAGE_MANAGER_SPEC.md`) and potentially build scripts (`build.ferra`) for this.

*   **Linking C Libraries into Ferra Programs**:
    *   **Manifest Configuration**: The Ferra project manifest should allow specifying dependencies on external C libraries.
        ```toml
        // Conceptual Ferra.toml snippet
        [package]
        name = "my_ferra_app"
        version = "0.1.0"

        [dependencies.c]
        my_c_lib = { version = "1.2", kind = "static" } // Example
        another_c_lib = { path = "libs/another", link_name = "another_custom_name" }
        ```
    *   **Build Scripts (`build.ferra`)**: For more complex scenarios (e.g., finding libraries using `pkg-config`, compiling C code as part of the Ferra build), a `build.ferra` script can be used. This script can programmatically instruct the Ferra compiler/linker.
        *   It can output directives to the compiler, similar to Cargo's build scripts:
            ```ferra
            // build.ferra (conceptual)
            fn main() {
                // Instruct Ferra to link against `libexample.a` or `libexample.so`
                println!("ferra:link-lib=static=example"); // Link libexample.a
                println!("ferra:link-lib=dylib=another");  // Link libanother.so or another.dll

                // Specify search paths for libraries
                println!("ferra:link-search=native=/opt/custom_libs/lib");
                println!("ferra:link-search=framework=/Library/Frameworks"); // e.g., for macOS frameworks

                // Rerun build script if a C header changes
                println!("ferra:rerun-if-changed=src/my_c_stuff/interface.h");

                // Example: Compiling a C file and linking it
                // let cc_compiler = find_c_compiler();
                // if cc_compiler.compile("src/my_c_stuff/helper.c").is_ok() { // Assuming compile outputs to OUT_DIR
                //    println!("ferra:link-lib=static=helper"); // Assuming output is libhelper.a in OUT_DIR
                //    println!("ferra:link-search=native=" + env::var("OUT_DIR").unwrap());
                // }
            }
            ```
        *   The exact `ferra:` directives (e.g., `ferra:link-lib`, `ferra:link-search`, `ferra:rerun-if-changed`) need to be formally specified (TBD FFI-C-6 and related to `PACKAGE_MANAGER_SPEC.md`).

*   **Linking Ferra Libraries into C/C++ Programs**:
    *   When Ferra code is compiled into a static or dynamic library, C/C++ projects will need to link against it.
    *   **Compiler Output**: The Ferra compiler (`lang build --lib`) should produce standard library file formats (`.a`/`.lib` for static, `.so`/`.dylib`/`.dll` for dynamic).
    *   **Header Files**: As discussed in Section 4.6, Ferra should generate C/C++ compatible header files declaring the exported `extern "C"` API. C/C++ projects would include this header.
    *   **Linker Instructions (for C/C++ build systems)**:
        *   The C/C++ build system (e.g., Make, CMake, Meson, Visual Studio) needs to be configured to:
            *   Add the directory containing the Ferra-generated header file to its include paths.
            *   Add the directory containing the Ferra-compiled library file to its library search paths.
            *   Specify the Ferra library name to the linker (e.g., `-lferra_lib_name`).
    *   **Documentation**: Ferra should provide clear documentation on how to integrate its compiled libraries into common C/C++ build systems.

## 8. Initial C++ Interoperability Considerations (v0.1)

While the primary focus of Ferra's v0.1 FFI is on interoperability with C, it's important to outline how C++ code can be interfaced with, given that C++ is a widely used language. For v0.1, Ferra will not attempt to directly interoperate with complex C++ ABI features like name mangling for classes, templates, or C++ exceptions. Instead, C++ interoperability will be achieved by leveraging the C ABI as a common bridge.

*   **Focus on C ABI Compatibility**:
    *   The most robust and portable way to call C++ code from Ferra (and for C++ to call Ferra code) is for the C++ code to expose a C-compatible interface.
    *   This means C++ functions intended for FFI with Ferra should be declared with `extern "C"`.
        ```cpp
        // C++ code (my_cpp_library.cpp)
        #include <vector>
        #include <string>
        #include <iostream>

        // Assume this is a C++ class we want to use from Ferra
        class MyCppObject {
        public:
            MyCppObject(const char* name) : name_(name) {}
            void greet() {
                std::cout << "Hello from C++ object: " << name_ << std::endl;
            }
            std::string getName() const { return name_; }
        private:
            std::string name_;
        };

        // C-style wrapper functions for the C++ object
        extern "C" {
            MyCppObject* create_my_cpp_object(const char* name) {
                return new MyCppObject(name);
            }

            void my_cpp_object_greet(MyCppObject* obj) {
                if (obj) obj->greet();
            }

            // For returning string data, C-compatible mechanisms must be used
            // (e.g., caller-provided buffer or callee-allocated C-string)
            void my_cpp_object_get_name(MyCppObject* obj, char* buffer, int buffer_len) {
                if (obj && buffer && buffer_len > 0) {
                    strncpy(buffer, obj->getName().c_str(), buffer_len - 1);
                    buffer[buffer_len - 1] = '\\0'; // Ensure null termination
                } else if (buffer && buffer_len > 0) {
                    buffer[0] = '\\0';
                }
            }

            void destroy_my_cpp_object(MyCppObject* obj) {
                delete obj;
            }
        }
        ```
    *   Ferra code would then declare and call these `extern "C"` wrapper functions as if they were standard C functions (using techniques from Section 3 and Section 4). The `MyCppObject*` would be treated as an opaque handle (`*mut c_void` or a newtyped pointer) in Ferra.

*   **Limitations for v0.1 C++ Interop**:
    *   **No Direct Class/Object Marshalling**: Ferra v0.1 will not have built-in support for directly creating, using, or marshalling C++ classes/objects that rely on C++ specific features (constructors, destructors, virtual methods, inheritance, templates).
    *   **No C++ Exception Handling**: Ferra FFI will not be able to catch C++ exceptions. If a C++ function called via an `extern "C"` wrapper can throw an exception, that exception must be caught within the C++ wrapper and translated into a C error code before returning to Ferra. Uncaught C++ exceptions crossing the FFI boundary typically lead to program termination.
    *   **No C++ Name Mangling Support**: Ferra will not attempt to call C++ functions by their mangled names. All C++ FFI entry points must use `extern "C"` to ensure C linkage.
    *   **No C++ Template Instantiation from Ferra**: Ferra cannot directly instantiate C++ templates. If specific template instantiations are needed, they must be exposed via `extern "C"` functions from the C++ side.
    *   **STL Types**: Standard C++ Library (STL) types like `std::string`, `std::vector`, `std::map` are not directly FFI-compatible. Data must be converted to C-compatible types (e.g., `const char*`, pointers to C arrays, C structs) at the `extern "C"` boundary.

*   **Using `extern "C"` in C++ to Interface with Ferra**:
    *   Similarly, if C++ code needs to call Ferra functions that have been exported with a C ABI (Section 4.1), the C++ code should declare these Ferra functions within an `extern "C"` block to prevent C++ name mangling issues when linking.
        ```cpp
        // C++ code calling Ferra
        extern "C" {
            // Assuming ferra_calculate_sum is exported from Ferra
            int32_t ferra_calculate_sum(int32_t a, int32_t b); 
        }

        void cpp_calls_ferra() {
            int32_t result = ferra_calculate_sum(10, 20);
            // ... use result ...
        }
        ```

For v0.1, relying on `extern "C"` provides a well-defined and widely supported method for basic interoperability between Ferra and C++. More advanced or direct C++ interop features are complex and deferred to future FFI enhancements.

## 9. Limitations and Future Work (Initial FFI)

The v0.1 FFI design for C and C++ provides a foundational capability for interoperability. However, it comes with certain limitations, and there are many potential areas for future enhancement as Ferra matures.

*   **Explicitly Out of Scope for v0.1 C/C++ FFI**:
    *   **Direct C++ Feature Interoperability**:
        *   No direct marshalling or understanding of C++ classes, objects, constructors, destructors (beyond what's managed via opaque handles and C wrappers).
        *   No direct support for C++ templates or template metaprogramming from Ferra.
        *   No direct handling of C++ exceptions across the FFI boundary (panics from Ferra are caught; C++ exceptions from C++ code called via C wrappers must be caught in C++).
        *   No direct support for C++ name mangling or calling C++ overloaded functions by their mangled names.
    *   **Advanced C Features**:
        *   Full, safe support for calling variadic C functions (e.g., `printf`). Initial recommendation is C wrappers.
        *   Direct, type-safe mapping for complex C preprocessor macros or highly platform-specific C constructs beyond standard ABI types.
        *   Deep integration with C bitfields beyond treating them as part of an opaque `#[repr(C)]` struct whose fields are accessed via C helper functions.
    *   **Tooling**:
        *   Automated generation of complex FFI bindings beyond basic C header files (e.g., tools like `cbindgen` or `swig` for other languages often offer more).
        *   Sophisticated build system support for complex C/C++ project structures or build systems (CMake, Meson) beyond basic library linking.
    *   **Other Languages**: FFI with languages other than C (and C++ via C ABI) is out of scope for this initial FFI design. This includes Rust, Python, JVM languages, .NET languages, etc., which are noted as future goals in `comprehensive_plan.md` (Module 4.1.4).

*   **Potential Future Enhancements**:
    *   **Improved C++ Interoperability**:
        *   Investigate safer, more direct ways to interact with C++ classes (e.g., bindings for simple classes, limited support for calling member functions).
        *   Potential for handling C++ exceptions at the FFI boundary if robust mechanisms can be found or developed.
        *   Tooling to help generate C++-side `extern "C"` wrappers for existing C++ codebases.
    *   **Enhanced Type System Support for FFI**:
        *   More comprehensive support for C types, including better handling of `long double`, C bitfields, and potentially C unions with type-safe access patterns in Ferra.
        *   Official support and robust type aliases for all POSIX and Windows C types in `core::ffi`.
    *   **Tooling and Automation**:
        *   More advanced binding generation tools (`ferra-bindgen`) capable of generating richer C/C++ headers or even wrapper code in other languages.
        *   Better integration with C/C++ build systems.
        *   Automated checks for FFI signature mismatches or potential unsafety.
    *   **Ergonomics and Safety**:
        *   More helper types and functions in the standard FFI module to reduce boilerplate and common errors (e.g., more robust `CString`, `CStr` implementations).
        *   Refined `unsafe` ergonomics for FFI.
        *   Improved diagnostic messages for FFI-related errors.
    *   **FFI for Other Languages**:
        *   Systematic design and implementation of FFI capabilities for other important languages as outlined in the project roadmap (Rust, Python, JVM, .NET). Each will have its own set of challenges and require dedicated design.
    *   **Variadic C Functions**: Revisit support for safely calling variadic C functions from Ferra.
    *   **ABI Stability for Ferra Itself**: As Ferra evolves towards v1.0, defining a stable ABI for Ferra functions (beyond just `extern "C"`) could become important if Ferra libraries are to be dynamically linked by other Ferra programs compiled with different compiler versions. This is a broader topic than just FFI with C.

The v0.1 FFI provides a crucial starting point. Future work will build upon this foundation to expand Ferra's interoperability and ease of use in mixed-language environments.

## 10. Open Questions / TBD

This section lists open questions and items marked as "To Be Determined" (TBD) during the drafting of this initial FFI specification for C/C++. These will need to be resolved as FFI design and implementation progresses.

*   **(FFI-C-1) Exact Syntax for FFI Declarations**:
    *   The precise syntax for `extern "C"` blocks for declaring external C functions (Section 3.1).
    *   The definitive syntax for attributes like `#[link_name = "..."]` (Section 3.1).
    *   The definitive syntax for exporting Ferra functions with C ABI, including `#[no_mangle]` and `#[export_name = "..."]` and their interaction (Section 4.1).
    *   These need to be aligned with or extensions to `SYNTAX_GRAMMAR_V0.1.md`.

*   **(FFI-C-2) `#[repr(...)]` Options for Data Layout Control**:
    *   A definitive list and specification of all `#[repr(...)]` attributes Ferra will support for FFI data layout control. This includes `#[repr(C)]` for structs/data classes (Section 3.2, 4.2, 5.4) and `#[repr(IntType)]` or `#[repr(C_Enum)]` for C-compatible enums (Section 3.2, 5.6).
    *   Specification for `#[repr(transparent)]` if used for opaque handles (Section 3.2).

*   **(FFI-C-3) String Marshalling Helper Functions/Types**:
    *   The strategy and API for helper functions or dedicated types (like conceptual `CString` / `CStr`) in the standard library or `core::ffi` module for managing null-termination, UTF-8 conversion, and ownership of strings passed to/from C (Section 5.3).

*   **(FFI-C-4) Panic Unwinding Strategy Across FFI Boundary**:
    *   The exact mechanism by which Ferra functions exported to C (`extern "C" fn`) will catch panics to prevent unwinding into C code (Section 4.4, 4.5). This includes specifying the behavior of a Ferra equivalent to `std::panic::catch_unwind` if developed, and the standard C error code returned.

*   **(FFI-C-5) Details for Callback Context Passing (`userdata`)**:
    *   Refining the patterns and safety guidelines for managing the lifetime and ownership of `userdata` (context pointers) passed alongside C function pointer callbacks, especially when the context is Ferra-managed memory or closures (Section 4.5).

*   **(FFI-C-6) Standard FFI Type Aliases**:
    *   Finalizing the list of C type aliases (e.g., `c_int`, `c_char`, `size_t`, `wchar_t`) to be provided in a standard Ferra FFI module (e.g., `core::ffi`) and ensuring their correct mapping for supported platforms (Section 3.2).

*   **(FFI-C-7) Variadic C Function Calls**:
    *   Re-evaluating if any level of support for calling variadic C functions from Ferra can be safely provided in v0.1, or confirming it's deferred (Section 3.1).

*   **(FFI-C-8) Ferra `Drop` Trait for FFI Resource Management**:
    *   The design and availability of a `Drop` trait in Ferra and how it can be leveraged to automate cleanup of C resources wrapped by Ferra types (mentioned in Section 3.4 and 3.6). This is tied to the broader Ferra language design for resource management.

*   **(FFI-C-9) C Unions and Bitfields**:
    *   Further investigation into the level of support or specific patterns for interoperating with C `union` types and C `struct` bitfields in v0.1, even if direct mapping is complex (Section 5.4).

*   **(FFI-C-10) Build System Directives for FFI**:
    *   The exact syntax and semantics of directives output by build scripts (e.g., `build.ferra`) to inform the Ferra build system about linker flags, library paths, etc. (e.g., `ferra:link-lib=...`) (Section 7).

Resolving these TBDs will be important for a complete and robust FFI implementation.
---
This document will specify the initial FFI capabilities for interacting with C and C++ code. 