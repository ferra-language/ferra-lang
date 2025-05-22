---
number: RFC-002
title: "Core Semantics and Types"
status: Draft
version: v0.4
authors: ["Amrit Doll <amritdoll@example.com>"]
last_reviewed: 2025-05-21
last_updated: 2025-05-21
---

# RFC-002: Core Semantics and Types

## Metadata
- **RFC Number**: 002
- **Title**: Core Semantics and Types
- **Status**: Draft
- **Version**: v0.4
- **Authors**: [Amrit Doll]
- **Created**: 2025-05-21
- **Last Updated**: 2025-05-21
- **Related RFCs**: 
  - RFC-001 (Syntax and Grammar)
  - RFC-003 (Ownership Model)
  - RFC-004 (Type System)
- **Supersedes**: None
- **Dependencies**: None

## Table of Contents
1. [Summary](#1-summary)
2. [Motivation](#2-motivation)
3. [Impact](#3-impact)
4. [Design Decisions](#4-design-decisions)
   1. [Execution Model](#41-execution-model)
   2. [Type System](#42-type-system)
   3. [Value Semantics](#43-value-semantics)
   4. [Type Inference](#44-type-inference)
   5. [Ownership & Borrowing](#45-ownership--borrowing)
5. [Drawbacks](#5-drawbacks)
6. [Security & Privacy](#6-security--privacy)
7. [Implementation Plan](#7-implementation-plan)
8. [Migration Strategy](#8-migration-strategy)
9. [Unresolved Questions](#9-unresolved-questions)
10. [Future Possibilities](#10-future-possibilities)
11. [References](#11-references)

## 1. Summary
This RFC defines the core semantics and type system for Ferra v0.1, establishing the foundational rules for program execution, type checking, and memory safety. It builds upon the syntax defined in RFC-001 and sets the stage for the ownership model in RFC-003.

## 2. Motivation
Ferra aims to provide a safe, performant, and ergonomic programming language that combines Python's ease of use with Rust's safety guarantees. The core semantics and type system are crucial for achieving these goals by:

1. Ensuring type safety and preventing runtime errors
2. Enabling efficient memory management without garbage collection
3. Supporting gradual typing for better developer experience
4. Providing clear error messages and helpful diagnostics

## 3. Impact
### 3.1 Developer Experience
- Clear value semantics and type system
- Helpful type inference and error messages
- Predictable ownership and borrowing rules
- Familiar control flow constructs

### 3.2 Ecosystem
- Standardized type system and semantics
- Consistent error handling patterns
- Clear guidelines for FFI and interop

### 3.3 Performance
- Efficient type inference
- Zero-cost abstractions where possible
- Predictable memory management

## 4. Design Decisions

### 4.1 Execution Model

#### 4.1.1 Program Entry
- Entry point: `fn main() -> Int` or `fn main() -> ()`
- Return value becomes process exit code (0 for success)
- Single-threaded, deterministic execution for v0.1

Example:
```ferra
fn main() -> Int {
    if !validate_input() {
        return 1  // Error exit code
    }
    process_data()
    return 0  // Success exit code
}
```

#### 4.1.2 Evaluation Strategy
- Eager evaluation by default
- Expression-oriented: many constructs yield values
- Statements without values produce `Unit` (`()`)

#### 4.1.3 Numeric Semantics
- Integer overflow: Wrapping in debug, trapping in release
- Float operations: IEEE 754 with strict NaN propagation
- Numeric conversions: Explicit only, no implicit widening/narrowing

Example:
```ferra
// Integer overflow
let x: Int = Int::MAX;
let y = x + 1;  // Debug: wraps to Int::MIN
               // Release: traps with overflow error

// Numeric conversions
let i: Int = 42;
let f: Float = Float::from(i);  // Explicit conversion
// let f2: Float = i;          // Error: implicit conversion
```

### 4.2 Type System

#### 4.2.1 Built-in Types
```ferra
// Scalar Types
let b: Bool = true;           // Boolean
let i: Int = 42;             // 64-bit signed integer
let f: Float = 3.14;         // 64-bit float
let c: Char = 'A';           // Unicode scalar
let s: String = "hello";     // UTF-8 string
let u: Unit = ();            // Unit type

// Compound Types
let t: (Int, String) = (1, "one");  // Tuple
let a: [Int] = [1, 2, 3];           // Array
let f: fn(Int) -> Bool = is_even;   // Function type
```

#### 4.2.2 Type Categories
1. **Scalar Types**
   - `Bool`: Boolean values
   - `Int`: 64-bit signed integer
   - `Float`: 64-bit IEEE 754
   - `Char`: Unicode scalar
   - `String`: UTF-8 string
   - `Unit`: Single value `()`

2. **Compound Types**
   - Tuples: `(T1, T2, ...)`
   - Arrays: `[T]`
   - Function types: `fn(T1, T2) -> R`
   - Data classes: `data Name { field: Type }`

3. **Generic Types** (Standard Library)
   - `Result<T, E>`: Error handling
   - `Option<T>`: Optional values

#### 4.2.3 Row Polymorphism Example
```ferra
// Example of row polymorphism with data classes
data User { id: Int, name: String }
data Customer { id: Int, name: String, email: String }

fn greet(person: { name: String }) -> String {
    "Hello, " + person.name
}

let user = User { id: 1, name: "Alice" }
let customer = Customer { id: 2, name: "Bob", email: "bob@example.com" }

// Both work due to row polymorphism
println(greet(user))      // Works: User has name field
println(greet(customer))  // Works: Customer has name field
```

### 4.3 Value Semantics

#### 4.3.1 Copy vs Move
```ferra
// Copy Types (default)
let x: Int = 5;
let y = x;  // x is copied, both valid

// Move Types (default)
let s1: String = "hello";
let s2 = s1;  // s1 moved to s2, s1 invalid

// Copy Data Classes
#[derive(Copy)]
data Point { x: Int, y: Int }
let p1 = Point { x: 1, y: 2 };
let p2 = p1;  // p1 copied, both valid
```

#### 4.3.2 Default Behaviors
| Type Category | Behavior | Notes |
|--------------|----------|-------|
| Scalar Types | Copy | `Int`, `Float`, `Bool`, `Unit` |
| Tuples | Copy if all fields Copy | `(Int, Bool)` |
| Arrays | Move | `[T]` is always Move |
| Strings | Move | Heap-allocated |
| Data Classes | Move | Can be marked Copy |

### 4.4 Type Inference
> See [DESIGN_TYPE_INFERENCE.md](../DESIGN_TYPE_INFERENCE.md#bidirectional-type-inference) for detailed implementation.

```ferra
// Example of bidirectional type inference with row polymorphism
fn process_user(user: {name: String, age: Int, ..}) -> Result<{name: String, age: Int, verified: Bool}, Error> {
    // Type inference propagates through Result
    let verified = verify_user(user)?;  // ? operator propagates Error
    Ok({...user, verified})
}

// Example of gradual typing with _
fn parse_config(path: String) -> Result<Config, Error> {
    let data = read_file(path)?;
    let config: _ = parse_json(data)?;  // Type inferred from usage
    Ok(config)
}
```

### 4.5 Ownership & Borrowing
> See [RFC-003: Ownership Model](./RFC-003_OWNERSHIP_MODEL.md#borrowing-rules) for complete borrowing rules.

      ```ferra
// Example of error propagation with Result
fn process_data(path: String) -> Result<Data, Error> {
    let file = File::open(path)?;  // Propagates Error if open fails
    let data = read_file(file)?;   // Propagates Error if read fails
    Ok(parse_data(data)?)         // Propagates Error if parse fails
}

// Example of error handling with custom error types
fn validate_user(user: User) -> Result<ValidatedUser, ValidationError> {
    if user.age < 18 {
        return Err(ValidationError::Underage);
    }
    if !user.email.contains('@') {
        return Err(ValidationError::InvalidEmail);
    }
    Ok(ValidatedUser(user))
}
```

## 5. Drawbacks
- Learning curve for ownership system
- Type inference complexity
- Performance overhead of safety checks
- Limited to Rank-1 polymorphism
- No explicit lifetime annotations

## 6. Security & Privacy
- Type safety prevents many bugs
- Ownership prevents memory issues
- No implicit type conversions
- Clear visibility boundaries
- Safe FFI through extern blocks

## 7. Implementation Plan
- **Phase 1a (Q3 2025)**
  - Basic type system implementation
  - Type inference engine
  - Initial borrow checker
  - Error reporting

- **Phase 1b (Q3 2025)**
  - Type-inference test harness
  - Row polymorphism tests
  - Borrow checker test suite
  - Common error scenario coverage

- **Phase 1.5 (Q3 2025)**
  - Integer overflow semantics (SEM-OVERFLOW-1)
  - Numeric conversion rules
  - Float operation specifications
  - Overflow test suite

- **Phase 2a (Q4 2025)**
  - Enhanced type inference
  - Improved error messages
  - Performance optimizations
  - FFI type safety

- **Phase 2b (Q4 2025)**
  - Array copy/move semantics (SEM-ARRAY-1)
  - Lifetime inference rules (SEM-LIFETIME-1)
  - Explicit lifetime preparation

## 8. Migration Strategy
The language is designed for gradual adoption:

1. **Type System Migration**
   - Start with basic types and inference
   - Add row polymorphism in Phase 1.5
   - Explicit lifetime annotations will be opt-in via `--explicit-lifetimes` flag
   - Migration tool will suggest lifetime annotations

2. **Error Handling Migration**
   - Start with basic `Result<T, E>` and `?` operator
   - Add custom error types in Phase 1.5
   - Migration guide for converting exception-based code

Example error handling:
```ferra
// Basic error handling
fn read_file(path: String) -> Result<String, IoError> {
    let file = File::open(path)?;  // ? propagates errors
    let content = file.read_to_string()?;
    Ok(content)
}

// Custom error types
data AppError {
    IoError(IoError),
    ParseError(ParseError),
    ValidationError(String)
}

fn process_data(input: String) -> Result<Data, AppError> {
    let file = File::open(input).map_err(AppError::IoError)?;
    let data = parse(file).map_err(AppError::ParseError)?;
    validate(data).map_err(AppError::ValidationError)
}
```

## 9. Unresolved Questions
### High Priority
- SEM-OVERFLOW-1: Integer overflow behavior (Phase 1.5)
- SEM-ARRAY-1: Array copy/move semantics (Phase 2b)
- SEM-COPY-1: Copy trait definition (Phase 1a)
- SEM-LIFETIME-1: Lifetime inference rules (Phase 2b)

### Medium Priority
- SEM-CONV-1: Numeric conversion rules
- SEM-ITER-1: Iterator protocol
- SEM-FLOAT-1: Float literal formats
- SEM-SIZE-1: Sized integer types

### Low Priority
- SEM-ARRAY-2: Fixed-size array syntax
- SEM-PTR-1: Pointer/reference types
- SEM-ASYNC-1: Async memory model
- SEM-TRAIT-1: Trait system design

## 10. Future Possibilities
- Higher-rank types
- Associated types
- Const generics
- Type-level programming
- Advanced pattern matching
- Custom operators
- Type-level metaprogramming

## 11. References
1. [RFC-001: Syntax and Grammar](./RFC-001_SYNTAX_GRAMMAR.md#41-lexical-structure)
2. [RFC-003: Ownership Model](./RFC-003_OWNERSHIP_MODEL.md#borrowing)
3. [RFC-004: Type System](./RFC-004_TYPE_SYSTEM.md#type-inference)
4. [CORE_SEMANTICS_V0.1.md](../CORE_SEMANTICS_V0.1.md#41-lexical-structure)
5. [DESIGN_TYPE_INFERENCE.md](../DESIGN_TYPE_INFERENCE.md#type-inference)
6. [OWNERSHIP_MODEL_PRINCIPLES_V0.1.md](../OWNERSHIP_MODEL_PRINCIPLES_V0.1.md#ownership)
7. [STDLIB_CORE_V0.1.md](../STDLIB_CORE_V0.1.md#error-handling)
