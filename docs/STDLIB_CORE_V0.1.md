# Ferra Core Standard Library (Minimal Set) v0.1

> **Status:** Initial Draft - Module 1.5 · Steps 1.5.1 & 1.5.2

## 1. Introduction

This document specifies the minimal set of core standard library APIs for Ferra v0.1. The goal is to provide essential functionalities for basic I/O, string manipulation, and common data structures that are fundamental for writing simple Ferra programs and for the self-hosting subset.

This initial version focuses on practicality and essential features. The standard library will be expanded significantly in later phases.

**Guiding Principles for v0.1:**

*   **Minimalism**: Include only what is strictly necessary for early utility and compiler self-hosting targets.
*   **Simplicity**: APIs should be straightforward and easy to use.
*   **Consistency**: Maintain consistency with Ferra's overall design philosophy and type system.
*   **Leverage Backend Runtimes**: Where appropriate, core functionalities (like memory allocation for strings/arrays, basic I/O) will rely on the runtime functions specified in `docs/BACKEND_LLVM_X86-64.md` (e.g., `ferra_alloc`, `ferra_panic`).
*   **Non-Panicking APIs**: All fallible operations in v0.1 return `Result<T, E>` or `Option<T>` to ensure predictable error handling, aligning with the diagnostics design. Panics are reserved for unrecoverable runtime errors.

## 2. Foundational I/O APIs

All I/O operations are initially synchronous for v0.1. The `IOError` data type is used for operations that can fail.

**`data IOError` Definition (v0.1):**
```ferra
data IOError {
    code: Int,      // OS-level error code or a Ferra-specific I/O error code.
    message: String // Human-readable error message.
}
```
This structure provides basic information about I/O failures. Its precise mapping to OS errors and the range of Ferra-specific codes are subject to runtime implementation details but the structure is fixed for v0.1 stdlib API.

### 2.1. Console Output

*   **`println(message: String) -> Result<(), IOError>`**: Prints the given string message to the standard output, followed by a newline character.
    *   **Returns**: `Ok(())` on success, or an `Err(IOError)` on failure.
    *   **Implementation Note**: This would typically map to a runtime call that interfaces with the target system's standard output (e.g., `libc::printf` or OS-specific console output functions).
    *   **Example**:
        ```ferra
        println("Hello, Ferra!");
        let name = "User";
        println("Welcome, " + name + "!"); // String concatenation to be supported
        ```

*   **`print(message: String) -> Result<(), IOError>`**: Prints the given string message to the standard output without a trailing newline.
    *   **Returns**: `Ok(())` on success, or an `Err(IOError)` on failure.
    *   **Example**:
        ```ferra
        print("Enter value: ");
        // ... read input ...
        ```

*   **`eprintln(message: String) -> Result<(), IOError>`**: Prints the given string message to the standard error output, followed by a newline character.
    *   **Returns**: `Ok(())` on success, or an `Err(IOError)` on failure.
    *   **Implementation Note**: Similar to `println`, but targets `stderr`.
    *   **Example**:
        ```ferra
        eprintln("Error: File not found.");
        ```

*   **`read_line() -> Result<String, IOError>`**: Reads a line of text from standard input.
    *   **Returns**: `Ok(String)` containing the input line (excluding the newline terminator) on success, or an `Err(IOError)` on failure.
    *   **Implementation Note**: This would map to a runtime call (e.g., `ferra_io_read`). The returned `String` is allocated by the runtime; the caller is responsible for its eventual deallocation as per Ferra's memory management rules to match the allocation ABI.
    *   **Example**:
        ```ferra
        print("Enter your name: ");
        let name = read_line();
        println("Hello, " + name + "!");
        ```

### 2.2. Future I/O Considerations (Out of Scope for v0.1)

*   Formatted printing (e.g., `printf`-style or string interpolation) - *Targeted for Phase 2*.
*   Standard input reading beyond `read_line()` (e.g., reading specific types).
*   File I/O.
*   Buffered I/O.
*   Asynchronous I/O operations.

## 3. Essential Data Structures

### 3.1. `String`

Ferra's `String` type is fundamental. Its core representation (`{data_ptr: *u8, len: Int}`) is defined by the backend/IR, but its user-facing API is part of the standard library.

*   **Properties**:
    *   UTF-8 encoded sequence of characters.
    *   Heap-allocated and dynamically sized.
    *   Immutable by default (modification operations typically produce new strings).

*   **Core Operations (v0.1)**:
    *   **`String::new() -> String`**: Creates a new, empty string.
    *   **`String::from_literal(literal: compile_time_string) -> String`**: (Conceptual) Compiler intrinsic for creating strings from string literals.
    *   **`len(s: String) -> Int`**: Returns the number of bytes in the string's UTF-8 representation.
        *   *(Note: `char_count` or similar for Unicode scalar values is a future consideration)*
    *   **`is_empty(s: String) -> Bool`**: Returns `true` if the string has zero length.
    *   **`+` (Concatenation Operator)**: `s1: String + s2: String -> String`
        *   Creates a new string by concatenating `s1` and `s2`.
        *   **Implementation Note**: Relies on runtime functions like `ferra_alloc` and `ferra_string_concat` (or equivalent for copying and joining byte data).
    *   **`String::from_int(value: Int) -> String`**: Converts an integer to its string representation.
        *   **Example**: `let s = String::from_int(123); // s is "123"`
        *   **Implementation Note**: Internally, this would likely rely on a runtime helper function (e.g., a future `ferra_rt::itoa`).

*   **Future `String` Enhancements**:
    *   Slicing (`s[start..end]`).
    *   Character iteration.
    *   Searching (`find`, `contains`).
    *   Modification (`push`, `insert`, `remove` - potentially on a mutable `StringBuilder` type).
    *   Case conversion.
    *   Splitting and joining.

### 3.2. `Vector<T>` (Dynamic Array / Vector)

A generic, dynamically-sized list of elements of type `T`. (Formerly `List<T>`)

*   **Properties**:
    *   Ordered collection.
    *   Elements are of the same type `T`.
    *   Heap-allocated and growable.
    *   Mutability of the vector and its elements will follow Ferra's ownership and borrowing rules.

*   **Core Operations (v0.1)**:
    *   **`Vector<T>::new() -> Vector<T>`**: Creates a new, empty vector.
    *   **`Vector<T>::with_capacity(capacity: Int) -> Vector<T>`**: Creates a new vector with an initial capacity hint. (Note: capacity is a hint; actual allocation is rounded up to the next power-of-two for efficient growth.)
    *   **`vector.len() -> Int`**: Returns the number of elements in the vector. (Assumes `&self` borrow)
    *   **`vector.is_empty() -> Bool`**: Returns `true` if the vector contains no elements. (Assumes `&self` borrow)
    *   **`vector.push(value: T)`**: Appends an element to the end of the vector.
        *   *(Requires a mutable borrow of the vector, i.e., `&mut self`)*
    *   **`vector.pop() -> Option<T>`**: Removes and returns the last element, or `None` if the vector is empty.
        *   *(Requires a mutable borrow of the vector, i.e., `&mut self`)*
    *   **`vector.get(index: Int) -> Option<&T>`**: Returns an optional reference to the element at the given index, or `None` if the index is out of bounds.
        *   *(Returns an immutable borrow, assumes `&self`)*
    *   **`vector.get_mut(index: Int) -> Option<&mut T>`**: Returns an optional mutable reference to the element at the given index.
        *   *(Requires a mutable borrow of the vector, i.e., `&mut self`)*
    *   **Literal Syntax**: `[elem1, elem2, ...]` (as defined in grammar/parser, AST constructs this).

*   **Implementation Notes**:
    *   The underlying memory layout will likely be `(data_ptr: *T, length: Int, capacity: Int)`, aligning with the backend `Vector` type.
    *   Operations will use runtime functions like `ferra_alloc`, `ferra_realloc`, `ferra_free` (was `ferra_dealloc`).

*   **Future `Vector<T>` Enhancements**:
    *   Iteration (`for item in my_vector`).
    *   Slicing.
    *   `insert`, `remove` at arbitrary index.
    *   Sorting, searching.
    *   Higher-order functions (`map`, `filter`, `reduce`).
    *   Unordered (iteration order not guaranteed for v0.1).

### 3.3. `Map<K, V>` (Basic Dictionary / Hash Map)

A generic collection of key-value pairs. For v0.1, this will be very minimal, primarily to support internal compiler needs if necessary, or as a placeholder for future expansion.

*   **Properties**:
    *   Keys `K` must be hashable and equatable (details TBD).
    *   Values `V` can be any type.
    *   Unordered (iteration order not guaranteed for v0.1).

*   **Core Operations (v0.1 - Tentative and Minimal)**:
    *   **`Map<K, V>::new() -> Map<K, V>`**: Creates a new, empty map.
    *   **`map.insert(key: K, value: V) -> Option<V>`**: Inserts a key-value pair. If the key already exists, updates the value and returns the old value. (Requires `&mut self` for `map`)
    *   **`map.get(key: K) -> Option<&V>`**: Retrieves an optional reference to the value associated with the key. (Requires `&self` for `map`)
    *   **`map.remove(key: K) -> Option<V>`**: Removes a key and its associated value, returning the value if it existed. (Requires `&mut self` for `map`)
    *   **`map.contains_key(key: K) -> Bool`**: Checks if the map contains the given key. (Requires `&self` for `map`)
    *   **`map.len() -> Int`**: Returns the number of key-value pairs. (Requires `&self` for `map`)
    *   **`map.is_empty() -> Bool`**: Checks if the map is empty. (Requires `&self` for `map`)

*   **Key Type Restriction (v0.1)**: For v0.1, `K` must be `String` or `Int`. Full support for user-defined types as keys depends on the future `Hash` and `Eq` traits.

*   **Future `Map<K, V>` Enhancements**:
    *   Literal syntax.
    *   Iteration over keys, values, or pairs.
    *   More sophisticated collision handling and performance tuning.
    *   Requirements for `K` (e.g., a `Hashable` trait).

## 4. Error Handling Types (Conceptual Placeholders)

While the full semantics are complex and tied to error handling RFCs, the standard library needs placeholders for `Result<T, E>` and `Option<T>` as they are often returned by core operations.

### 4.1. `Option<T>`

Represents an optional value: either `Some(T)` or `None`.

*   **Variants**:
    *   `Some(value: T)`: Contains a value.
    *   `None`: Represents the absence of a value.

*   **Key Conceptual Operations (v0.1 - for understanding, full API TBD)**:
    *   `is_some(opt: Option<T>) -> Bool`
    *   `is_none(opt: Option<T>) -> Bool`
    *   Used by `Vector::pop`, `Map::get`, etc.
*   **IR Representation**: Conceptually an enum-like data class, represented in the IR as a two-field tagged union (`{i64 tag, Payload}` as specified in IR Spec §5).

### 4.2. `Result<T, E>`

Represents either a success `Ok(T)` or an error `Err(E)`.

*   **Variants**:
    *   `Ok(value: T)`: Contains a success value.
    *   `Err(error: E)`: Contains an error value.

*   **Key Conceptual Operations (v0.1 - for understanding, full API TBD)**:
    *   Used by operations that can fail (e.g., future file I/O, parsing strings to numbers).
    *   The `?` operator (defined in `CORE_SEMANTICS_V0.1.md`) is designed to work with `Result`-like types.
*   **IR Representation**: Similar to `Option<T>`, `Result<T,E>` is conceptually an enum-like data class, represented in the IR as a two-field tagged union (`{i64 tag, Payload}` as specified in IR Spec §5).

## 5. Type Conversions and Traits (Minimal)

### 5.1. Basic Conversions

*   Explicit conversion functions might be provided where safe and common, e.g., `Int::to_float(val: Int) -> Float`.
*   The `as` keyword behavior for primitive type casting is defined in the IR and backend, but stdlib might provide more idiomatic wrappers if needed.
*   *(Related TBD: See "TYPE-LIT-DEFAULTS" open tag in the type inference design document regarding default types for literals and conversion behaviors.)*

### 5.2. Placeholder for Core Traits (Future)

While full trait definitions are for a later phase, certain concepts are implicit:

*   **Equality**: Types used as keys in `Map` will need a notion of equality.
*   **Hashing**: Types used as keys in `Map` will need a hashing mechanism.
*   **Display/Debug**: For `println` and debugging, a way to convert types to strings will be needed (more advanced than just `String` itself).

## 6. Core FFI Utilities Module (e.g., `core::ffi`)

To support interoperability with C as defined in `docs/FFI_C_CPP.md`, the standard library will provide a foundational module for FFI. This module will primarily contain type aliases for C standard types, ensuring portability and clarity in FFI declarations.

### 6.1. C Type Aliases

The FFI module will define type aliases corresponding to common C fixed-width and general-purpose integer and floating-point types. These ensure that Ferra code can declare C function signatures with types that match the C ABI on the target platform.

*   **Integer Aliases**:
    *   `c_char`: Platform's `char` (might be `i8` or `u8`).
    *   `c_schar`: Platform's `signed char` (typically `i8`).
    *   `c_uchar`: Platform's `unsigned char` (typically `u8`).
    *   `c_short`: Platform's `short` (typically `i16`).
    *   `c_ushort`: Platform's `unsigned short` (typically `u16`).
    *   `c_int`: Platform's `int` (typically `i32`).
    *   `c_uint`: Platform's `unsigned int` (typically `u32`).
    *   `c_long`: Platform's `long` (platform-dependent, e.g., `i32` or `i64`).
    *   `c_ulong`: Platform's `unsigned long` (platform-dependent, e.g., `u32` or `u64`).
    *   `c_longlong`: Platform's `long long` (typically `i64`).
    *   `c_ulonglong`: Platform's `unsigned long long` (typically `u64`).
    *   `size_t`: Platform's `size_t` (an unsigned integer type, e.g., `u64` on 64-bit platforms).
    *   `ptrdiff_t`: Platform's `ptrdiff_t` (a signed integer type for pointer differences).
    *   (Includes aliases for `int8_t`, `uint8_t`, `int16_t`, etc., from `<stdint.h>` if not directly represented by Ferra's `iX`/`uX` types in all FFI contexts).

*   **Floating-Point Aliases**:
    *   `c_float`: Platform's `float` (typically `f32`).
    *   `c_double`: Platform's `double` (typically `f64`).
    *   *(Note: `c_long_double` might be omitted for v0.1 due to portability issues, as mentioned in `FFI_C_CPP.md`)*

*   **Other C Aliases**:
    *   `c_void`: An opaque type representing C's `void` when used with pointers (e.g., `*mut c_void`). This could be an empty struct or enum in Ferra.
    *   `c_bool`: Platform's `bool` (from C99 `_Bool`, typically mapping to an integer).

*   **Example Usage**:
    ```ferra
    import core::ffi::{c_int, c_char}; // Conceptual import path

    extern "C" {
        fn c_strlen(s: *const c_char) -> c_int;
        // fn c_printf(format: *const c_char, ...); // Variadic TBD for FFI
    }
    ```

### 6.2. Pointer Utilities (Conceptual for v0.1)

While extensive FFI helper types like `CString` are future work (TBD FFI-C-3), some very basic pointer utilities might reside here or in a dedicated `core::ptr` module.

*   `ptr::null<T>() -> *const T`: Returns a null const raw pointer of type `*const T`.
*   `ptr::null_mut<T>() -> *mut T`: Returns a null mutable raw pointer of type `*mut T`.
*   Methods on raw pointers like `is_null()` (as used in examples in `FFI_C_CPP.md`).

Further utilities, especially for string marshalling (such as the conceptual `CString` and `CStr` types from `FFI_C_CPP.md` TBD FFI-C-3), would be specified here or in a dedicated submodule like `core::ffi::string` as they are designed. The type aliases in section 6.1 address TBD FFI-C-7 from the FFI design document.

This module provides the necessary building blocks for writing FFI declarations that are both clear and portable across different C ABIs where Ferra is supported.

## 7. Data Parallelism and GPU Computing Support (Conceptual for v0.1 Extensions)

To support data-parallel operations and GPU offloading as detailed in `DATA_PARALLEL_GPU.md`, the standard library will be extended with concepts for parallel iteration and GPU interaction. The full specification of these APIs will reside in a dedicated document or a later version of the stdlib spec, but their conceptual inclusion begins here.

### 7.1. Parallel Iterators (Conceptual)

*   **Purpose**: Provide a way to process collections in a data-parallel manner on the CPU, primarily targeting SIMD vectorization.
*   **Key Methods (on relevant collections like `Vector<T>`)**:
    *   `par_iter() -> ParallelIterator<Item=&T>`: Returns a parallel iterator over immutable references to elements.
    *   `par_iter_mut() -> ParallelIterator<Item=&mut T>`: Returns a parallel iterator over mutable references to elements.
*   **`ParallelIterator` Trait (Conceptual)**:
    *   `for_each(closure: fn(Item) -> Unit)`: Applies the closure to each element in parallel. The closure body must adhere to restrictions suitable for parallel execution (e.g., limited side-effects, no inter-iteration dependencies for simple vectorization).
    *   Other methods like `map`, `filter`, `reduce` could be added in the future.
*   **Details**: See `DATA_PARALLEL_GPU.md` (Section 2).

### 7.2. GPU Support Module (`core::gpu` - Conceptual)

*   **Purpose**: Provide APIs for managing GPU devices, contexts, memory buffers, kernel compilation/launch, and host-device synchronization.
*   **Key Components (Conceptual - see `DATA_PARALLEL_GPU.md` Section 4.5 for more details)**:
    *   `gpu::Context`: Represents a connection to a GPU device.
    *   `GpuBuffer<T>`: Represents a data buffer allocated in GPU device memory.
        *   Methods for allocation, deallocation, writing from host, reading to host.
    *   Kernel Launch APIs: Functions to launch `#[gpu]` compiled kernels, passing `GpuBuffer`s and other parameters.
    *   Synchronization Primitives: For host to wait on GPU completion.
    *   GPU Intrinsics: Functions callable only from `#[gpu]` code (e.g., `global_id_x()`, `workgroup_barrier()`).
*   This module will abstract underlying GPU compute APIs (e.g., Vulkan for SPIR-V execution).

## 8. Open Questions / TBD (StdLib Core v0.1) (Renumbering from old Section 7)

| Tag             | Issue                                                                                             |
|-----------------|---------------------------------------------------------------------------------------------------|
| STDLIB-STR-ITER | String character iteration mechanism and API.                                                       |
| STDLIB-COLL-INIT| Efficient initialization of collections (e.g., `Vector::from_iter(...)`, `collect` patterns).       |
| STDLIB-MAP-KEY  | Precise requirements for `Map` keys (e.g., `Hash` and `Eq` traits).                                 |
| STDLIB-ERR-API  | Detailed API for `Option<T>` and `Result<T, E>` beyond conceptual variants.                       |
| STDLIB-NUM-CONV | Comprehensive numeric conversion strategy (`From`/`Into`-like traits vs. explicit functions).       |
| STDLIB-MEM-EXPOSE| Should raw memory allocation/deallocation functions (`ferra_alloc`, etc.) be exposed directly in std? |
| STDLIB-CHAR-1   | Definition and API for a `Char` type (Unicode scalar value).                                      |
| STDLIB-IO-ASYNC-1| Design for asynchronous I/O operations (mentioned in §2.2).                                      |
| STDLIB-FFI-STRING-HELPERS | Availability and API of `CString`/`CStr`-like types for FFI string marshalling. |

---

This document outlines the core principles and minimal API surface for Ferra's standard library v0.1.

This initial specification lays the groundwork for Ferra's standard library. It will be iteratively refined and expanded based on language evolution, implementation experience, and community feedback. 