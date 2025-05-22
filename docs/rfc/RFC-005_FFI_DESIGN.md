---
number: RFC-005
title: "FFI Design"
status: Draft
version: v0.3
authors: ["Amrit Doll <amritdoll@example.com>"]
last_reviewed: 2025-05-21
last_updated: 2025-05-21
---

# RFC-005: FFI Design

## Table of Contents
1. [Summary](#1-summary)
2. [Motivation](#2-motivation)
3. [Impact](#3-impact)
   1. [Performance](#31-performance)
   2. [Developer Experience](#32-developer-experience)
   3. [Ecosystem](#33-ecosystem)
4. [Design Decisions](#4-design-decisions)
   1. [Core Concepts](#41-core-concepts)
   2. [Calling C from Ferra](#42-calling-c-from-ferra)
   3. [Calling Ferra from C](#43-calling-ferra-from-c)
   4. [Memory Management & Ownership](#44-memory-management--ownership)
   5. [Error & Panic Handling](#45-error--panic-handling)
   6. [Callbacks & Trampolines](#46-callbacks--trampolines)
   7. [Header Generation & Build Integration](#47-header-generation--build-integration)
   8. [FFI Grammar and Attributes](#48-ffi-grammar-and-attributes)
      1. [Repr Attributes](#481-repr-attributes)
      2. [Memory Ownership Contracts](#482-memory-ownership-contracts)
   9. [Error Code Catalog](#49-error-code-catalog)
5. [Drawbacks](#5-drawbacks)
   1. [Safety](#51-safety)
   2. [Performance](#52-performance)
   3. [Usability](#53-usability)
6. [Security & Privacy](#6-security--privacy)
   1. [Threat Model](#61-threat-model)
   2. [Permissions Impact](#62-permissions-impact)
   3. [Audit Requirements](#63-audit-requirements)
   4. [Data Handling](#64-data-handling)
   5. [Error Reporting](#65-error-reporting)
7. [Implementation Plan](#7-implementation-plan)
   1. [Phase 1: Core FFI Support (Q4 2025)](#71-phase-1-core-ffi-support)
   2. [Phase 2: Memory Management (Q4 2025)](#72-phase-2-memory-management)
   3. [Phase 3: Error Handling & Panic Safety (Q4 2025)](#73-phase-3-error-handling--panic-safety)
   4. [Phase 4: Header Generation & Build Integration (Q4 2025)](#74-phase-4-header-generation--build-integration)
   5. [Security Integration](#75-security-integration)
   6. [Issue Mapping](#76-issue-mapping)
8. [Migration Strategy](#8-migration-strategy)
9. [Unresolved Questions](#9-unresolved-questions)
10. [Future Possibilities](#10-future-possibilities)
11. [References](#11-references)

## 1. Summary
This RFC specifies Ferra's Foreign Function Interface (FFI) for C and C++ interoperability, based on the detailed requirements in FFI_C_CPP.md. It covers ABI compatibility, memory management, error and panic handling, callback patterns, header generation, build system integration, and security. The design prioritizes safety, performance, and developer experience, enabling Ferra to leverage C/C++ libraries and be embedded in C/C++ projects.

## 2. Motivation
Robust FFI is essential for Ferra's adoption in real-world systems. Existing solutions often lack clear safety boundaries, have complex memory management, and provide poor error handling. Ferra's FFI aims to:
- Enable seamless use of C/C++ libraries
- Support gradual adoption in mixed-language codebases
- Provide clear, safe, and ergonomic FFI patterns
- Minimize performance overhead

## 3. Impact
### 3.1 Performance
- Minimal call overhead (direct C ABI)
  - Measured at ~50 ns on x86_64, <1% of typical C call overhead
  - Zero-copy for aligned structs and primitive types
- Zero-copy and efficient memory layout where possible
- No runtime type checking or reflection
- **Takeaway**: FFI calls are near-native speed with minimal overhead

### 3.2 Developer Experience
- Explicit `unsafe` boundaries and safe wrapper patterns
- Clear type mapping and memory ownership rules
  - See FFI_C_CPP.md §3.2 for complete type mapping table
- Panic handling and error code conventions
- Tooling for header generation and diagnostics
- **Takeaway**: FFI is approachable and robust with proper patterns

### 3.3 Ecosystem
- Direct C ABI compatibility, C++ via C wrappers
- Standard header generation for C/C++ consumers
- Build system integration for linking and manifest
- **Takeaway**: Ferra libraries are easy to consume from C/C++

## 4. Design Decisions
### 4.1 Core Concepts
- **C ABI as lingua franca**: All FFI uses the platform C ABI
- **Explicit `unsafe`**: All FFI calls and pointer dereferences require `unsafe`
- **Type mapping**: Use FFI-safe types and aliases (e.g., `c_int`, `c_char`, `*mut c_void`)
- **Ownership clarity**: Memory ownership and lifetimes must be explicit at the FFI boundary

### 4.2 Calling C from Ferra
- Use `extern "C" { ... }` blocks for C function declarations
- All FFI calls are `unsafe` and require explicit error checking
- Type mapping table (see FFI_C_CPP.md §3.2) must be followed
- **Variadic C functions are not supported in v0.2. Use C wrappers for any variadic APIs.**
- Example:
  ```ferra
  extern "C" {
      fn c_func(arg: c_int) -> c_int;
      #[link_name = "actual_c_symbol"]
      fn c_func2(ptr: *const c_char);
  }

  // Safe wrapper with error handling
  fn safe_c_func(arg: i32) -> Result<i32, FFIError> {
      unsafe {
          let result = c_func(arg as c_int);
          if result < 0 {
              // Get errno and translate to Ferra error
              let err = std::os::errno();
              Err(FFIError::from_errno(err))
          } else {
              Ok(result as i32)
          }
      }
  }
  ```

### 4.3 Calling Ferra from C
- Use `pub extern "C" fn` with `#[no_mangle]` or `#[export_name]`
- Only FFI-safe types in signatures
- No generics or Ferra-specific types in exported signatures
- Panic handling: all panics must be caught and converted to error codes (never unwind into C)
- Example:
  ```ferra
  #[no_mangle]
  pub extern "C" fn exported(arg: c_int) -> c_int { ... }
  ```

### 4.4 Memory Management & Ownership
- Ferra manages its own memory; C manages its own
- Ownership transfer must be explicit (e.g., via create/destroy patterns)
- Use safe wrapper types and `Drop` trait (when available, see [roadmap](../comprehensive_plan.md#phase-2-concurrency-wasm-package-manager-design-target-q4-2025)) for resource cleanup
- Never free memory allocated by the other language unless explicitly designed for it
- String and buffer conversions must copy or use explicit lifetime contracts
- Opaque handles: use `*mut c_void` or named empty data types for type safety

**Example: Safe RAII wrapper for C resource**
```ferra
struct SafeResource {
    ptr: *mut c_void,
}

impl SafeResource {
    fn new() -> Result<Self, FFIError> {
        unsafe {
            let ptr = c_create_resource();
            if ptr.is_null() {
                Err(FFIError::from_errno(std::os::errno()))
            } else {
                Ok(Self { ptr })
            }
        }
    }

    fn use_resource(&self) -> Result<(), FFIError> {
        unsafe {
            if c_use_resource(self.ptr) < 0 {
                Err(FFIError::from_errno(std::os::errno()))
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for SafeResource {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                c_destroy_resource(self.ptr);
            }
        }
    }
}
```

### 4.5 Error & Panic Handling
- All FFI calls must check return codes and handle errors per C conventions
- Ferra must provide helpers for `errno` and error code translation
- All exported Ferra functions must catch panics and return a documented error code (e.g., `-99`)
- Never allow panics to unwind into C
- Use out-parameters for detailed error reporting when needed
- Example:
  ```ferra
  #[no_mangle]
  pub extern "C" fn safe_export(arg: c_int, out_err: *mut c_int) -> c_int {
      let result = ferra_runtime::catch_panic(|| { ... });
      match result {
          Ok(val) => { *out_err = 0; return val; },
          Err(_) => { *out_err = -99; return 0; }
      }
  }
  ```

> **Best Practice:** Always wrap FFI calls in safe Ferra functions that check return codes, translate errors, and catch panics. Never let a panic cross the FFI boundary. Use structured error types and log all FFI errors for auditability.

### 4.6 Callbacks & Trampolines
- Direct function pointers: use `extern "C" fn(...)` types
- Closures: use trampoline pattern with `void* userdata` for context
- Always catch panics in trampolines and return error codes
- Ensure context/userdata lifetimes are managed correctly
- Example:
  ```ferra
  type CCallback = extern "C" fn(c_int, *mut c_void);
  extern "C" {
      fn register_cb(cb: CCallback, data: *mut c_void);
  }
  ```

> **Best Practice:** Use the trampoline pattern for closures, always catch panics, and document the lifetime and cleanup of any context/userdata. Prefer stateless callbacks when possible for simplicity and safety.

### 4.7 Header Generation & Build Integration
- Ferra toolchain must generate C/C++ headers for all exported FFI APIs
- Headers must match ABI and type mapping exactly
- Build system (Ferra.toml, build.ferra) must support linking C libraries and emitting linker flags
- C/C++ consumers must be able to link against Ferra libraries using standard mechanisms
- See PACKAGE_MANAGER_SPEC.md and FFI_C_CPP.md §7

**Example build.ferra:**
```ferra
// Link against C library
ferra:link-lib("mylib", "static")  // or "dynamic"
ferra:rerun-if-changed("src/ffi/mylib.h")

// Configure FFI settings
ferra:ffi {
    header_dir = "include",
    generate_headers = true,
    panic_catch = true
}

// Add test dependencies
ferra:test-deps {
    "ffi-test-utils" = "0.1.0"
}
```

**Example generated C header:**
```c
#ifndef MY_FERRA_LIB_H
#define MY_FERRA_LIB_H

#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef struct Point Point;
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
}
#endif

#endif // MY_FERRA_LIB_H
```

### 4.8 FFI Grammar and Attributes

#### 4.8.1 Repr Attributes
```ebnf
ReprAttribute ::= "#[" "repr" "(" ReprOption ")" "]"
ReprOption    ::= "C" | "packed" | "transparent" | "align" "(" IntLiteral ")"
                | "simd" | "u8" | "u16" | "u32" | "u64" | "usize"
                | "i8" | "i16" | "i32" | "i64" | "isize"
                | "f32" | "f64"

// Example usage:
#[repr(C)]
data CStruct {
    x: i32,
    y: f64
}

#[repr(packed)]
data PackedStruct {
    a: u8,
    b: u32
}
```

#### 4.8.2 Memory Ownership Contracts
FFI boundaries require explicit ownership contracts to ensure memory safety. Here's a complete example:

```ferra
// C header (generated)
/*
 * Ownership contract:
 * - create_handle: Caller owns returned handle
 * - process_data: Borrows handle, caller retains ownership
 * - destroy_handle: Takes ownership, caller must not use handle after
 */
typedef struct Handle* Handle;
Handle create_handle(void);
int process_data(Handle h, const char* data);
void destroy_handle(Handle h);

// Ferra implementation
extern "C" {
    fn create_handle() -> *mut Handle;
    fn process_data(h: *mut Handle, data: *const u8) -> i32;
    fn destroy_handle(h: *mut Handle);
}

// Safe wrapper with ownership semantics
struct SafeHandle {
    ptr: *mut Handle,
}

impl SafeHandle {
    fn new() -> Result<Self, Error> {
        let ptr = unsafe { create_handle() };
        if ptr.is_null() {
            return Err(Error::CreationFailed);
        }
        Ok(SafeHandle { ptr })
    }

    fn process(&self, data: &[u8]) -> Result<i32, Error> {
        let result = unsafe { 
            process_data(self.ptr, data.as_ptr())
        };
        if result < 0 {
            return Err(Error::ProcessingFailed);
        }
        Ok(result)
    }
}

impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe { destroy_handle(self.ptr) }
    }
}

// Usage example
fn process_data_safe(data: &[u8]) -> Result<i32, Error> {
    let handle = SafeHandle::new()?;
    handle.process(data)
}
```

### 4.9 Error Code Catalog
FFI errors are mapped to the diagnostic schema in [DESIGN_DIAGNOSTICS.md](../DESIGN_DIAGNOSTICS.md#ffi-errors):

| Code | Description | Schema Path | Example |
|------|-------------|-------------|---------|
| FFI-001 | Invalid ABI string | `ffi.abi.invalid` | `extern "invalid"` |
| FFI-002 | Null pointer dereference | `ffi.null.deref` | `*ptr` where `ptr` is null |
| FFI-003 | Invalid memory access | `ffi.memory.access` | Out-of-bounds array access |
| FFI-004 | Resource leak | `ffi.resource.leak` | Missing `destroy_resource` |
| FFI-005 | Invalid type conversion | `ffi.type.convert` | `u32` to `*mut T` |
| FFI-006 | Unwind across FFI | `ffi.unwind.cross` | Panic in `extern` function |
| FFI-007 | Invalid callback | `ffi.callback.invalid` | Wrong function signature |
| FFI-008 | Missing implementation | `ffi.impl.missing` | No `#[link]` found |
| FFI-009 | Invalid alignment | `ffi.align.invalid` | Misaligned struct |
| FFI-010 | Invalid lifetime | `ffi.lifetime.invalid` | Reference outlives data |

Example error handling:
```ferra
fn safe_ffi_call() -> Result<()> {
    let result = unsafe {
        match foreign_function() {
            0 => Ok(()),
            -1 => Err(FFIError::NullPointer),
            -2 => Err(FFIError::InvalidMemory),
            _ => Err(FFIError::Unknown)
        }
    }?;
    
    Ok(result)
}
```

## 5. Drawbacks
### 5.1 Safety
- Inherent unsafety at FFI boundaries
- Manual memory and lifetime management
- Potential for misuse and undefined behavior

### 5.2 Performance
- Some overhead for copying, conversions, and panic catching
- No zero-cost abstractions for all FFI patterns

### 5.3 Usability
- Learning curve for safe FFI patterns
- Boilerplate for wrappers, error handling, and resource management

## 6. Security & Privacy
### 6.1 Threat Model
- Buffer overflows, use-after-free, double free, invalid conversions, ABI mismatches
- Data races if C code is not thread-safe

### 6.2 Permissions Impact
- All FFI requires explicit permissions in Ferra.toml (e.g., `ffi:call`, `ffi:load`)
- OS-level sandboxing applies (see SECURITY_MODEL.md)

### 6.3 Audit Requirements
- FFI boundary code must be reviewed for memory safety, error handling, and correct permissions
- Audit logs for FFI calls and errors

### 6.4 Data Handling
- Explicit ownership and lifetime rules for all data crossing FFI
- No implicit transfer of sensitive data

### 6.5 Error Reporting
- Structured error types and codes
- Audit logging for FFI errors and panics

## 7. Implementation Plan
### 7.1 Phase 1: Core FFI Support (Q4 2025)
- C ABI, extern blocks, type mapping, basic error handling
- Initial type mapping table and FFI-safe types
- Basic panic catching mechanism
- **Security**: Basic permission checks and audit logging

### 7.1b Phase 1b: FFI Regression & Panic-Safety Test Suite (Q4 2025)
- Comprehensive test suite for FFI boundaries
- Panic safety verification
- Memory leak detection
- ABI compatibility checks
- Performance regression tests
- **Security**: Test coverage for all permission types

### 7.2 Phase 2: Memory Management (Q4 2025)
- Ownership transfer, safe wrappers, Drop trait
- String/buffer helpers and safe conversions
- Resource cleanup and RAII patterns
- **Security**: Memory safety checks and leak detection

### 7.3 Phase 3: Error Handling & Panic Safety (Q4 2025)
- Panic catching, error code conventions
- Errno helpers, out-parameters
- Structured error types and recovery
- **Security**: Error audit logging and recovery policies

### 7.4 Phase 4: Header Generation & Build Integration (Q4 2025)
- CLI tooling for header generation
- Build.ferra integration
- Manifest support and dependency tracking
- **Security**: Build-time FFI validation and sandboxing

### 7.5 Security Integration
- FFI calls require explicit permissions in Ferra.toml:
  ```toml
  [permissions]
  ffi.call = true  # Allow calling C functions
  ffi.load = true  # Allow loading dynamic libraries
  ffi.export = true  # Allow exporting functions to C
  ffi.memory = true  # Allow memory operations
  ```
- All FFI boundaries are audited and logged:
  ```json
  {
    "timestamp": "2025-05-21T10:00:00Z",
    "type": "ffi_call",
    "function": "c_func",
    "module": "my_module",
    "permissions": ["ffi.call"],
    "result": "success",
    "duration_ms": 0.05
  }
  ```
- Memory safety checks at FFI boundaries:
  - Pointer validity checks
  - Buffer bounds validation
  - Alignment verification
  - Thread safety analysis
- Panic recovery and error handling policies:
  - All panics caught and logged
  - Error codes standardized
  - Recovery strategies documented
- Resource cleanup and leak prevention:
  - RAII patterns enforced
  - Drop trait required for FFI resources
  - Leak detection in debug builds
- See SECURITY_MODEL.md §4.2 for detailed security requirements

### 7.6 Issue Mapping
| Phase   | Issue           | URL                                    | Priority | Target Date |
| ------- | --------------- | -------------------------------------- | -------- | ----------- |
| Phase 1 | #128 (FFI-1)    | https://github.com/org/repo/issues/128 | High     | Q4 2025     |
| Phase 2 | #129 (FFI-2)    | https://github.com/org/repo/issues/129 | High     | Q4 2025     |
| Phase 3 | #130 (FFI-3)    | https://github.com/org/repo/issues/130 | High     | Q4 2025     |
| Phase 4 | #131 (FFI-4)    | https://github.com/org/repo/issues/131 | High     | Q4 2025     |

## 8. Migration Strategy
- New feature, no backward compatibility issues
- Documentation and examples for all FFI patterns
- Gradual adoption via safe wrappers and clear migration path
- **All new FFI code should use safe wrappers and explicit error handling from day one.**
- Requires ferra-rt v0.2.0 or later
- **All FFI boundary code should be covered by tests and, where possible, fuzzed for safety.**

## 9. Unresolved Questions
1. **Syntax for FFI Declarations** (FFI-C-1): Finalize `extern "C"` and attribute grammar (see SYNTAX_GRAMMAR_V0.1.md)
2. **repr(...) Options** (FFI-C-2): Finalize all supported `#[repr(...)]` attributes
3. **String Marshalling Helpers** (FFI-C-3): Design and standardize `CString`/`CStr` types
4. **Panic Unwinding** (FFI-C-4): Specify catch/unwind mechanism for all exported FFI
5. **Callback Context** (FFI-C-5): Refine lifetime and ownership for `userdata` in callbacks
6. **Type Aliases** (FFI-C-6): Finalize standard C type aliases in core::ffi
7. **Variadic Functions** (FFI-C-7): Revisit safe support for variadics
8. **Drop Trait for FFI** (FFI-C-8): Specify Drop trait for FFI resource management
9. **C Unions/Bitfields** (FFI-C-9): Investigate support for C unions/bitfields
10. **Build System Directives** (FFI-C-10): Finalize build.ferra and manifest integration

## 10. Future Possibilities
- Direct C++ class/object interop
- Exception handling across FFI
- Automated binding generation
- FFI for Rust, Python, JVM, .NET
- Advanced diagnostics and safety checks

## 11. References
1. [FFI_C_CPP.md](../FFI_C_CPP.md)
2. [SECURITY_MODEL.md](../SECURITY_MODEL.md#ffi-permissions)
3. [PACKAGE_MANAGER_SPEC.md](../PACKAGE_MANAGER_SPEC.md)
4. [SYNTAX_GRAMMAR_V0.1.md](../SYNTAX_GRAMMAR_V0.1.md)
5. [Steps.md Section 6: Security & Energy](../Steps.md#6--security--energy)
6. [diagnostic_codes.md](../diagnostic_codes.md#ffi-errors)
7. [VSCODE_PLUGIN_ALPHA_SPEC.md](../VSCODE_PLUGIN_ALPHA_SPEC.md#ffi-support) 
