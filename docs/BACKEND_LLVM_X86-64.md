# LLVM Backend for Ferra (x86-64) v0.1

> **Status:** Draft — Module 1.4 · Step 1.4.1 & 1.4.2
>
> Specifies the LLVM backend infrastructure requirements and IR to LLVM IR conversion logic for the x86-64 target architecture.

---

## 1 Overview

This document describes the Ferra compiler's LLVM backend implementation targeting x86-64 architectures. LLVM provides:

* **Mature optimization pipeline** – Well-tested, production-grade optimization passes
* **Reliable code generation** – Battle-tested x86-64 backend with support for latest ISA features
* **Cross-platform support** – Foundation for future architecture targets
* **Comprehensive tooling** – Integration with debuggers, profilers, and existing toolchains

The backend translates Ferra's IR (as defined in `IR_SPECIFICATION.md`) to LLVM IR, which is then passed to LLVM's optimization pipelines and code generators.

This document primarily details the x86-64 target. However, many of the principles regarding Ferra IR to LLVM IR conversion, LLVM infrastructure requirements, runtime support function ABI, and CLI option structure also serve as a foundational basis for other Ferra backends that leverage LLVM, such as ARM-64 and Apple Bitcode generation, as detailed in `BACKEND_EXPANDED_TARGETS.md`.

---

## 2 LLVM Infrastructure Requirements

### 2.1 LLVM Version & Dependencies

* **LLVM Version**: 17.0.0 or later (for all required features)
* **License Compatibility**: LLVM 17 is Apache-2.0 with LLVM-exceptions, compatible with Ferra's licensing model (see [§ 8.1 License compatibility](#8-1-license-compatibility))
* **Core Components**:
  * LLVM Core libraries
  * Clang (for integration with C/C++ ecosystem)
  * lld (LLVM linker)
* **Build Integration**:
  * CMake 3.20+
  * Ninja build system
* **Distribution**:
  * Pre-built LLVM binaries for developer workflows
  * Statically linked LLVM libraries in production builds
  * Windows: `/MT` by default (static CRT), `/MD` (dynamic CRT) as optional configuration
    * Static CRT: Standalone executables with minimal dependencies
    * Dynamic CRT: Suitable for apps integrating with other Windows components

### 2.2 Integration Architecture

```
┌───────────────┐   ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│               │   │               │   │               │   │               │
│ Ferra IR      │──▶│ LLVM IR       │──▶│ LLVM Opts     │──▶│ LLVM Backend  │
│ (SSA Quads)   │   │ (Module)      │   │ (Passes)      │   │ (CodeGen)     │
│               │   │               │   │               │   │               │
└───────────────┘   └───────────────┘   └───────────────┘   └───────────────┘
                           │                                        │
                           ▼                                        ▼
                    ┌───────────────┐                       ┌───────────────┐
                    │               │                       │               │
                    │ Bitcode Files │                       │ Object Files  │
                    │ (.bc)         │                       │ (.o)          │
                    │               │                       │               │
                    └───────────────┘                       └───────────────┘
                                                                   │
                                                                   ▼
                                                            ┌───────────────┐
                                                            │               │
                                                            │ Executables   │
                                                            │ (.exe/.bin)   │
                                                            │               │
                                                            └───────────────┘
```

### 2.3 Target Architecture Support

* **Initial Target**: x86-64 Linux (`x86_64-unknown-linux-gnu`)
* **Windows Support**: x86-64 Windows (`x86_64-pc-windows-msvc`)
* **macOS Support**: x86-64 macOS (`x86_64-apple-darwin`) once CI has Clang-17 integration
* **Position-Independent Code**: Default for shared libraries and executables
* **Code Model**: `small` (< 2GB code + data) default, `medium` available for larger applications

### 2.4 LLVM API Requirements

* **C++ API**: Primary integration using LLVM's C++ API
* **Library Organization**:
  * `Core` – Data structures, IR, passes
  * `Analysis` – Pass management
  * `CodeGen` – Target-specific code generation
  * `Target` – Machine code generation for x86-64
  * `MC` – Machine code emission
  * `Object` – Object file manipulation
  * `Linker` – Module linking

---

## 3 Ferra IR to LLVM IR Conversion

### 3.1 Type Mapping

| Ferra Type | LLVM Type | Notes |
|------------|-----------|-------|
| `Unit ()` | `void` | Used for functions with no return value |
| `Bool` | `i1` | 1-bit integer for boolean values |
| `Int` | `i64` | Default integer size (64-bit) |
| `Float` | `double` | Default floating-point (64-bit) |
| `Char` | `i32` | Unicode codepoint (32-bit) |
| `String` | `{ i8*, i64 }` | Pointer+length slice representation[^1] |
| `(T1, T2, ...)` (Tuple) | `{ T1, T2, ... }` | Struct with unnamed fields |
| `[T]` (Array/Vector) | `{ %T*, i64, i64 }` | Pointer to data, length, capacity. Typically a pointer to a heap-allocated struct like `%Array.T` defined in §3.2. The type `%T` is the LLVM mapping of Ferra type T. |
| `UserDefinedType` (data class) | LLVM Named Struct | Ferra `data` classes are mapped to LLVM named struct types (e.g., `%MyStruct = type { field1_type, field2_type }`) based on their field definitions from the IR type table. |
| `fn(T1, T2) -> R` | Function pointer | See function representation |
| `Result<T, E>` | `{ i64, %T_or_E_payload }` | Tag + payload (see Result ABI note below)[^2]. `%T_or_E_payload` represents the storage for T or E in the union. |

[^1]: String representation uses the `{ i8*, i64 }` slice format for efficient operations. This canonical representation is used throughout the compiler for consistent handling of string data.

[^2]: Result<T, E> ABI: Uses a discriminated union with a 64-bit tag (0 for success, error code otherwise) followed by value payload in a union layout. The tag field always comes first for consistent ABI across platforms. The payload part (e.g. `%T_or_E_payload`) would be a union typically sized to the max of T and E, or handled via `i8*` and casts if types are opaque.

### 3.2 String & Array Representation

Ferra strings and arrays are heap-allocated with a consistent memory layout:

```
┌───────┬───────┬─────────────────┐
│ length│ cap   │ data...         │
│ (i64) │ (i64) │ (elements)      │
└───────┴───────┴─────────────────┘
```

In LLVM IR, this is represented as:

```llvm
%String = type { i64, i64, [0 x i8] }
%Array.T = type { i64, i64, [0 x %T] }
```

With operations for creation, access, and manipulation handled by runtime functions.

### 3.3 Memory Management

* **Stack allocation**: Used for temporaries and smaller values
* **Heap allocation**: Managed through runtime functions:
  * `@ferra_alloc(i64 %size) -> i8*` – Allocate memory
  * `@ferra_free(i8* %ptr, i64 %size) -> void` – Deallocate memory (aligns with `STDLIB_CORE_V0.1.md`)
  * `@ferra_realloc(i8* %ptr, i64 %old_size, i64 %new_size) -> i8*` – Resize allocation
  * `@ferra_panic(i32 %file_id, i32 %line, i32 %col, i8* %msg)` – Fatal error handling with source location and program termination, standardized ABI for all error sources

### 3.4 Function Representation

Functions are represented with LLVM function declarations:

```llvm
define [linkage] [return_type] @name([param_type %param_name, ...]) {
  ; Function body
}
```

For external functions, declarations are used:

```llvm
declare [return_type] @name([param_type, ...])
```

Calling conventions:
* Linux/macOS: SystemV ABI (`ccc`)
* Windows: Microsoft x64 ABI (`win64cc`)
* Ferra v0.1 forbids varargs; will lower to `...` with SysV in future RFC

### 3.5 Control Flow Mapping

| Ferra IR | LLVM IR | Notes |
|----------|---------|-------|
| `br` | `br label %target` | Unconditional branch |
| `br_cond` | `br i1 %cond, label %true, label %false` | Conditional branch |
| `ret` | `ret <type> %value` | Return with value |
| `phi` | `%result = phi <type> [%val1, %block1], [%val2, %block2], ...` | PHI node for SSA form |

### 3.6 Instruction Mapping

| Ferra IR Op | LLVM IR | Notes |
|-------------|---------|-------|
| `add` | `add nsw` | Signed addition with wrap |
| `sub` | `sub nsw` | Signed subtraction with wrap |
| `mul` | `mul nsw` | Signed multiplication with wrap |
| `div` | `sdiv` | Signed division |
| `rem` | `srem` | Signed remainder |
| `fadd` | `fadd` | Floating-point addition |
| `fsub` | `fsub` | Floating-point subtraction |
| `fmul` | `fmul` | Floating-point multiplication |
| `fdiv` | `fdiv` | Floating-point division |
| `eq` | `icmp eq` / `fcmp oeq` | Integer/float equality |
| `ne` | `icmp ne` / `fcmp one` | Integer/float inequality |
| `lt` | `icmp slt` / `fcmp olt` | Integer/float less than |
| `le` | `icmp sle` / `fcmp ole` | Integer/float less than or equal |
| `gt` | `icmp sgt` / `fcmp ogt` | Integer/float greater than |
| `ge` | `icmp sge` / `fcmp oge` | Integer/float greater than or equal |
| `and` | `and` | Bitwise AND |
| `or` | `or` | Bitwise OR |
| `xor` | `xor` | Bitwise XOR |
| `shl` | `shl` | Shift left |
| `shr` | `ashr` | Arithmetic shift right |
| `cast` | Various conversion ops | See conversion matrix below |
| `call` | `call` | Function invocation |
| `alloca` | `alloca` | Stack allocation |
| `load` | `load` | Memory load |
| `store` | `store` | Memory store |
| `gep` | `getelementptr` | Address calculation for structs/arrays |

### 3.7 Type Conversion Matrix

| From → To | Bool | Int | Float | Char | Notes |
|-----------|------|-----|-------|------|-------|
| Bool→ | - | `zext` | `uitofp` | `zext` | 0/1 → int |
| Int→ | `icmp ne` | - | `sitofp` | `trunc`/`zext` | !=0 for bool |
| Float→ | `fcmp one` | `fptosi` | - | n/a | !=0.0 for bool |
| Char→ | `icmp ne` | `zext`/`trunc` | `uitofp` | - | !=0 for bool |

Note: When unsigned integer types are introduced in the future, comparison operators will use the `u` prefix variants (`icmp ult`, `icmp ule`, etc.) instead of the signed `s` prefix variants shown above.

### 3.8 Semantic Tag Integration

* LLVM metadata namespace string = `ferra`
* Tags from the IR (see `IR_SEMANTIC_TAGS.md`) can inform LLVM-level optimizations by mapping specific IR tags to LLVM metadata or instruction attributes (e.g., `!ferra.inline` → function attribute `inlinehint`, `!ferra.vectorize` → loop metadata).
* For persistence and AI tooling consumption, the LLVM backend is responsible for embedding the complete CBOR-encoded semantic tag map (as defined in `IR_SEMANTIC_TAGS.md`) into the designated custom section (`.note.ai` for ELF, `ferra.ai` for Wasm, as specified in `IR_SEMANTIC_TAGS.md` §2) of the output object file or executable. This is handled separately from LLVM's internal metadata system used for optimizations.

### 3.9 Error Handling (`?` Operator)

The postfix `?` operator for error propagation is lowered to:

```llvm
%result = call @function()
%is_error = extractvalue %result, 0
%is_error_bool = icmp ne i64 %is_error, 0
br i1 %is_error_bool, label %propagate_error, label %continue

propagate_error:
    %error_value = extractvalue %result, 0
    %return_error = insertvalue { i64, %ReturnType } undef, i64 %error_value, 0
    ret { i64, %ReturnType } %return_error

continue:
    %success_value = extractvalue %result, 1
    ; Continue with %success_value
```

---

## 4 Runtime Support

### 4.1 Core Runtime Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `ferra_init` | `void ()` | Runtime initialization |
| `ferra_shutdown` | `void ()` | Runtime cleanup |
| `ferra_alloc` | `i8* (i64)` | Memory allocation |
| `ferra_free` | `void (i8*, i64)` | Memory deallocation |
| `ferra_string_new` | `i8* (i8*, i64)` | Create string from bytes |
| `ferra_string_concat` | `i8* (i8*, i8*)` | Concatenate strings |
| `ferra_array_new` | `i8* (i64, i64)` | Create array with element size |
| `ferra_panic` | `void (i32, i32, i32, i8*)` | Fatal error handling with source location |

### 4.2 Linking Model

* **Static linking**: Default for standalone executables
* **Dynamic linking**: Available for integrating with system libraries
* **Custom sections**: Support for `.note.ai` and other Ferra-specific sections in generated objects
* **Symbol naming**: Based on fully qualified path with mangling for generics

### 4.3 Command Line Interface

The compiler CLI provides options to control the backend:

```
ferra build [options] <source>

Backend Options:
  --target=<triple>     Target architecture (default: host)
  --opt-level=[0-3|s|z] Optimization level (default: 2)
  --emit=[ir|bc|obj]    Emission target (default: exe)
  --debug               Include debug information
  --debug-level=[1-3]   Debug information detail level
```

Note: Specialized analysis tools integrated into the compilation pipeline, such as the energy profiler (see `ENERGY_PROFILER.md` Section 5.1), may introduce their own set of command-line flags (e.g., `--energy-profile`) for enabling and configuring their specific analyses.

For Windows targets, PDB debug information is generated using the `/DEBUG:GHASH` flag with lld-link.

See the upcoming CLI specification in Module 1.7 for complete flag grammar and usage.

---

## 5 Optimizations

### 5.1 LLVM Optimization Passes

| Phase | Passes | Purpose |
|-------|--------|---------|
| Scalar | Mem2Reg, SCCP, GVN, InstCombine | Basic scalar optimizations |
| Loop | LoopUnroll, LoopVectorize | Loop transformations |
| Interprocedural | Inliner, GlobalOpt | Cross-function optimizations |
| Backend | FunctionAttrs, TailCallElim | Target-specific optimizations |
| Ferra-specific | FerraDeinitPass, InlineAlways | Ownership model integration |

In addition to these standard optimization passes, the LLVM pipeline in Ferra can accommodate custom *analysis passes*. These passes gather information about the code without necessarily transforming it. An example is the energy profiling pass (detailed in `ENERGY_PROFILER.md`), which analyzes LLVM IR to estimate energy consumption.

### 5.2 Optimization Levels

| Level | LLVM Preset | Purpose |
|-------|-------------|---------|
| `-O0` | `O0` | Debugging builds (minimal optimization) |
| `-O1` | `O1` | Basic optimization with quick compile times |
| `-O2` | `O2` | Production default - good perf/size balance |
| `-O3` | `O3` | Maximum performance optimization |
| `-Os` | `Os` | Size optimization |
| `-Oz` | `Oz` | Maximum size reduction |

### 5.3 Semantic Tag Integration

* Tags from the IR (see `IR_SEMANTIC_TAGS.md`) inform optimization decisions
* `inline` → LLVM `inlinehint` attribute
* `vectorize` → LLVM metadata for loop vectorization
* Targets with specialized hardware (AVX-512, etc.) get target-specific optimizations

---

## 6 Debug Information

### 6.1 DWARF Generation

* DWARF v5 debug format for x86-64 targets
* Source location mapping (preserved from Ferra AST → IR → LLVM IR)
* Variable information including types
* Inlining tracking
* Line tables for debugging

### 6.2 Debugger Integration

* LLDB debugging support
* GDB compatibility
* VSCode debugging protocol integration
* Symbol PDB generation for Windows targets using `/DEBUG:GHASH` for lld-link
* Debug information preservation flags:
  * `--keep-note` preserves `.note.ai` sections in release builds
  * `--strip-debug` removes debug sections but keeps symbols

---

## 7 Implementation Strategy

### 7.1 Incremental Implementation Order

1. Basic type mapping and module structure
2. Memory operations (alloca, load, store)
3. Arithmetic and comparison operators
4. Control flow constructs
5. Function calls and parameter passing
6. Runtime library integration

### 7.2 Testing Strategy

* Unit tests for each LLVM IR generation component
* End-to-end tests with known input/output
* Instruction-by-instruction verification
* Cross-reference against reference compiler implementation
* Performance benchmarking for regression detection

---

## 8 Extensions and Future Work

### 8.1 License Compatibility

LLVM's Apache-2.0 with LLVM-exceptions license is fully compatible with Ferra's licensing model. This ensures that:
- Distribution rights are clear for both open-source and commercial applications
- Patent grants are handled appropriately 
- Contributions to Ferra that include LLVM-derived code maintain license compatibility

### 8.2 Planned Extensions

* **SIMD Support**: Vector types and operations for data-parallel code
* **Exception Handling**: Implementation of Ferra's error handling in LLVM
* **Thread-Local Storage**: Support for thread-local variables
* **Profile-Guided Optimization**: PGO feedback integration
* **Hardware-Specific Optimizations**: AVX-512, BMI, etc.

---

## 9 Open Questions / TBD

| Tag | Issue |
|-----|-------|
| LLVM-VER-1 | Final minimum LLVM version required |
| LLVM-LAYOUT-1 | ABI compatibility for struct layouts |
| LLVM-STRING-1 | Final string representation (fat pointer vs object) |
| LLVM-ERROR-1 | Error propagation mechanism (unwinding vs explicit returns) |
| LLVM-SYNC-1 | Concurrency primitives implementation |
| BACKEND-TARGET-DARWIN-1 | macOS support timeline and specific requirements |
| BACKEND-RT-1 | Runtime ABI and function signatures:<br>`@ferra_init() -> void`<br>`@ferra_shutdown() -> void`<br>`@ferra_alloc(i64 %size) -> i8*`<br>`@ferra_free(i8* %ptr, i64 %size) -> void`<br>`@ferra_realloc(i8* %ptr, i64 %old_size, i64 %new_size) -> i8*`<br>`@ferra_string_new(i8* %bytes, i64 %len) -> i8*`<br>`@ferra_string_concat(i8* %str1, i8* %str2) -> i8*`<br>`@ferra_array_new(i64 %elem_size, i64 %capacity) -> i8*`<br>`@ferra_panic(i32 %file_id, i32 %line, i32 %col, i8* %msg) -> void` | 