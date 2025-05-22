---
number: RFC-003
title: "Ownership and Borrowing Model"
status: Draft
version: v0.5
authors: ["Amrit Doll <amritdoll@example.com>"]
last_reviewed: 2025-05-21
last_updated: 2025-05-21
---

# RFC-003: Ownership and Borrowing Model

## Metadata
- **RFC Number**: 003
- **Title**: Ownership and Borrowing Model
- **Status**: Draft
- **Version**: v0.5
- **Authors**: [Amrit Doll]
- **Created**: 2025-05-21
- **Last Updated**: 2025-05-21
- **Related RFCs**: 
  - RFC-001 (Syntax and Grammar)
  - RFC-002 (Core Semantics)
  - RFC-004 (Concurrency Model)
- **Supersedes**: None
- **Dependencies**: RFC-002 (Core Semantics)

## Table of Contents
1. [Summary](#1-summary)
2. [Motivation](#2-motivation)
3. [Impact](#3-impact)
4. [Design Decisions](#4-design-decisions)
   1. [Core Ownership Rules](#41-core-ownership-rules)
   2. [Move Semantics](#42-move-semantics)
   3. [Copy Semantics](#43-copy-semantics)
   4. [Borrowing Rules](#44-borrowing-rules)
   5. [Lifetimes](#45-lifetimes)
   6. [Integration with Language Features](#46-integration-with-language-features)
5. [Drawbacks](#5-drawbacks)
6. [Security & Privacy](#6-security--privacy)
7. [Implementation Plan](#7-implementation-plan)
8. [Migration Strategy](#8-migration-strategy)
9. [Unresolved Questions](#9-unresolved-questions)
10. [Future Possibilities](#10-future-possibilities)
11. [References](#11-references)

## 1. Summary
This RFC defines Ferra's ownership and borrowing model, establishing compile-time memory safety without garbage collection. The model combines Rust's safety guarantees with Python's ease of use, featuring clear rules, positive-first error messages, and sensible defaults.

## 2. Motivation
Memory safety is crucial for building reliable software. Traditional approaches like garbage collection or manual memory management have significant drawbacks:
- Garbage collection adds runtime overhead and unpredictability
- Manual memory management is error-prone and difficult to use correctly
- Reference counting can lead to cycles and performance issues

Ferra's ownership model aims to:
1. Prevent memory errors at compile time
2. Provide predictable performance without GC
3. Make memory safety approachable for developers
4. Enable safe concurrency through ownership rules

## 3. Impact
### 3.1 Developer Experience
- Clear ownership and borrowing rules
- Helpful, positive-first error messages
- Sensible defaults and type inference
- Familiar control flow with safety guarantees

### 3.2 Ecosystem
- Standardized memory safety patterns
- Consistent error handling
- Clear guidelines for FFI and interop

### 3.3 Performance
- Zero-cost abstractions
- Predictable memory management
- No runtime overhead for safety checks
- Borrow checking latency < 50ms
- Minimal compile-time impact (< 5% on clean builds)
- Memory safety without GC
- Compile-time ownership verification
- Zero runtime checks in release mode
- Efficient memory layout for owned types
- Optimized borrow checker data structures

## 4. Design Decisions

### 4.1 Core Ownership Rules

#### 4.1.1 Single Ownership
- Each value has exactly one owner at a time
- Ownership is transferred through moves
- Owner is responsible for dropping the value

Example:
```ferra
let s1 = String::from("hello"); // s1 owns the string
let s2 = s1;                    // ownership moves to s2
// println(s1);                 // Error: s1 no longer owns the string
println(s2);                    // OK: s2 owns the string
```

#### 4.1.2 Scope-Bound Lifetime
- Values live as long as their owner's scope
- Automatic cleanup when owner goes out of scope
- Prevents use-after-free and memory leaks

Example:
```ferra
{
    let s = String::from("hello"); // s owns the string
    println(s);                    // OK: s is in scope
} // s goes out of scope, string is dropped
// println(s);                    // Error: s is no longer in scope
```

### 4.2 Move Semantics

#### 4.2.1 Default Behavior
- Move semantics for resource-managing types
- Ownership transfer on assignment and function calls
- Original variable becomes invalid after move

Example:
```ferra
fn process(s: String) {
    println(s);
} // s is dropped here

let s1 = String::from("hello");
process(s1);        // s1 is moved into process
// println(s1);     // Error: s1 was moved
```

#### 4.2.2 Move-Only Types
- `String`, `[T]`, and user-defined `data` types
- Cannot be copied, only moved
- Ensures proper resource cleanup

### 4.3 Copy Semantics

#### 4.3.1 Copy Types
- Built-in scalar types (`Int`, `Float`, `Bool`, `Unit`)
- Tuples of `Copy` types
- Arrays of `Copy` types (shallow copy)
- References (`&T`, `&mut T`)

Example:
```ferra
let x: Int = 5;
let y = x;          // x is copied to y
println(x);         // OK: x is still valid
println(y);         // OK: y has its own copy
```

#### 4.3.2 Copy Trait
- Types can be marked as `Copy` if all fields are `Copy`
- Enables implicit copying instead of moves
- Must be safe to bitwise copy

Example:
```ferra
#[derive(Copy)]
data Point { x: Int, y: Int }

let p1 = Point { x: 1, y: 2 };
let p2 = p1;        // p1 is copied to p2
println(p1.x);      // OK: p1 is still valid
```

### 4.4 Borrowing Rules

#### 4.4.1 Shared References (`&T`)
- Multiple shared references allowed
- Read-only access
- Cannot coexist with mutable references

Example:
```ferra
let s = String::from("hello");
let r1 = &s;        // Shared borrow
let r2 = &s;        // Another shared borrow
println(*r1);       // OK: Can read through r1
println(*r2);       // OK: Can read through r2
// *r1 = "world";   // Error: Cannot mutate through &T
```

#### 4.4.2 Mutable References (`&mut T`)
- Only one mutable reference at a time
- Read-write access
- Cannot coexist with any other references

Example:
```ferra
var s = String::from("hello");
let r = &mut s;     // Mutable borrow
*r = String::from("world"); // OK: Can mutate through &mut T
// let r2 = &s;     // Error: Cannot borrow while r is active
```

#### 4.4.3 Borrow Checker Implementation
The borrow checker is implemented as a static analysis pass that tracks the state of all borrows. Here's the core implementation:

```ferra
// Core borrow checker data structures
type BorrowKind = Shared | Mutable
type BorrowState = {
    kind: BorrowKind,
    span: Span,
    is_active: bool
}

type BorrowEnv = {
    borrows: Map<Expr, Vec<BorrowState>>,
    active_mutables: Set<Expr>
}

// EBNF for borrow expressions
borrow_expr = "&" ["mut"] expr
reference_type = "&" ["mut"] type

// Core borrow checking algorithm
fn check_borrow(expr: Expr, env: &mut BorrowEnv) -> Result<(), BorrowError> {
    match expr {
        BorrowExpr { kind: Shared, target, span } => {
            // FERR_BORROW_001: Cannot borrow as shared while mutably borrowed
            if env.active_mutables.contains(&target) {
                return Err(BorrowError {
                    code: "FERR_BORROW_001",
                    message: "Cannot borrow as shared while mutably borrowed",
                    span,
                    suggestion: "Wait for mutable borrow to end or clone the value"
                });
            }
            env.borrows.entry(target).or_default().push(BorrowState {
                kind: Shared,
                span,
                is_active: true
            });
        }
        BorrowExpr { kind: Mutable, target, span } => {
            // FERR_BORROW_002: Cannot borrow as mutable while borrowed
            if let Some(borrows) = env.borrows.get(&target) {
                if borrows.iter().any(|b| b.is_active) {
                    return Err(BorrowError {
                        code: "FERR_BORROW_002",
                        message: "Cannot borrow as mutable while borrowed",
                        span,
                        suggestion: "Ensure no other borrows are active"
                    });
                }
            }
            env.active_mutables.insert(target.clone());
            env.borrows.entry(target).or_default().push(BorrowState {
                kind: Mutable,
                span,
                is_active: true
            });
        }
    }
    Ok(())
}
```

##### Diagnostic Code Catalog
The borrow checker uses the following error codes, defined in `DESIGN_DIAGNOSTICS.md`:

| Code | Schema Path | Description | Example |
|------|-------------|-------------|---------|
| FERR_BORROW_001 | `errors.borrow.shared_while_mutable` | Cannot borrow as shared while mutably borrowed | ```ferra<br>let mut x = 5;<br>let r1 = &mut x;  // Mutable borrow<br>let r2 = &x;      // Error: shared borrow while mutable borrow exists<br>``` |
| FERR_BORROW_002 | `errors.borrow.mutable_while_borrowed` | Cannot borrow as mutable while borrowed | ```ferra<br>let mut x = 5;<br>let r1 = &x;      // Shared borrow<br>let r2 = &mut x;  // Error: mutable borrow while shared borrow exists<br>``` |
| FERR_BORROW_003 | `errors.borrow.field_while_parent_borrowed` | Cannot borrow field while parent is borrowed | ```ferra<br>let mut user = User { ... };<br>let r1 = &user;           // Parent borrow<br>let r2 = &mut user.name;  // Error: field borrow while parent borrowed<br>``` |
| FERR_BORROW_004 | `errors.borrow.unsafe_with_borrows` | Cannot have active borrows during unsafe operation | ```ferra<br>let mut x = 5;<br>let r = &mut x;<br>unsafe {  // Error: active borrow during unsafe block<br>    process_data(r);<br>}<br>``` |

Each error code maps to a JSON schema in `DESIGN_DIAGNOSTICS.md` that defines:
- Error message format and severity
- Suggested fixes with code examples
- Related documentation links
- Common patterns to avoid
- Migration strategies

#### 4.4.4 Partial Borrows Analysis
The borrow checker implements a sophisticated disjoint path analysis to enable granular field borrowing:

```ferra
// Disjoint path analysis pseudocode
fn analyze_disjoint_paths(expr: Expr) -> Set<Path> {
    match expr {
        FieldAccess { target, field } => {
            let mut paths = analyze_disjoint_paths(target);
            paths.insert(Path::new(field));
            paths
        }
        ArrayIndex { target, index } => {
            let mut paths = analyze_disjoint_paths(target);
            paths.insert(Path::new(index));
            paths
        }
        // ... other cases
    }
}

fn are_paths_disjoint(p1: &Path, p2: &Path) -> bool {
    match (p1, p2) {
        (Path::Field(f1), Path::Field(f2)) => f1 != f2,
        (Path::Index(i1), Path::Index(i2)) => i1 != i2,
        _ => true
    }
}
```

This analysis enables:
1. Independent borrowing of different fields
2. Mutable borrows of one field while others are shared
3. Compile-time verification of disjoint access

#### 4.4.5 FFI Safety and Ownership
Foreign function interfaces require careful handling of ownership boundaries. The syntax for extern blocks is defined in RFC-001:

```ebnf
# From RFC-001_SYNTAX_GRAMMAR.md
ExternBlock = "extern" StringLiteral "{" ExternItem* "}"
ExternItem = "fn" IDENTIFIER "(" ExternParam* ")" [ "->" Type ]
ExternParam = [ "mut" ] Type
```

Example usage:
```ferra
extern "C" {
    // Raw pointers in extern blocks are exempt from normal borrow rules
    fn process_data(data: *mut u8, len: usize) -> i32;
}

fn safe_wrapper(data: &mut [u8]) -> i32 {
    // Compiler verifies no active borrows before unsafe block
    let ptr = data.as_mut_ptr();
    let len = data.len();
    
    // FERR_BORROW_004: Cannot have active borrows during unsafe operation
    unsafe {
        // Compiler ensures data is not borrowed elsewhere
        process_data(ptr, len)
    }
}

// Lifetime annotations for FFI
fn with_lifetime<'a>(data: &'a mut [u8]) -> &'a [u8] {
    unsafe {
        // Compiler verifies lifetime safety
        std::slice::from_raw_parts(data.as_ptr(), data.len())
    }
}
```

Safety rules for FFI:
1. Raw pointers in `extern` blocks are exempt from normal borrow rules
2. `unsafe` blocks require no active borrows of affected data
3. Compiler verifies borrow state before and after unsafe operations
4. Lifetime annotations required for references passed to foreign code
5. Automatic borrow scope extension across FFI boundaries

### 4.5 Lifetimes

#### 4.5.1 Lexical Lifetimes
- References valid within their lexical scope
- Compiler ensures references don't outlive data
- Prevents dangling references

Example:
```ferra
let r: &String;
{
    let s = String::from("hello");
    r = &s;         // r borrows s
    println(*r);    // OK: s is still alive
} // s is dropped
// println(*r);     // Error: r would be dangling
```

#### 4.5.2 Non-Lexical Lifetimes (NLL)
Non-lexical lifetimes allow borrows to end before their lexical scope, enabling more precise borrow scoping and improving ergonomics without sacrificing safety.

```ferra
// Example 1: Borrow ends after use
fn process(data: &[u8]) -> Result<(), Error> {
    let first = &data[0];  // Borrow starts
    if first == 0 {
        return Err(Error::InvalidData);  // Borrow ends here
    }
    // Can borrow data again here
    let rest = &data[1..];
    process_rest(rest)
}

// Example 2: Borrow ends in conditional
fn find_first(data: &[u8], target: u8) -> Option<usize> {
    for i in 0..data.len() {
        let current = &data[i];  // Borrow starts
        if *current == target {
            return Some(i);  // Borrow ends here
        }
        // Can borrow data again in next iteration
    }
    None
}

// Example 3: NLL with control flow
fn process_items(items: &[Item]) -> Result<(), Error> {
    let first = &items[0];  // Borrow starts
    if first.is_valid() {
        // Borrow ends here
        return process_valid(first);
    }
    // Can borrow items again here
    let rest = &items[1..];
    process_rest(rest)
}
```

#### 4.5.3 Lifetime Inference
- Compiler infers lifetimes in common cases
- No explicit annotations needed for v0.1
- Makes ownership system more approachable

### 4.6 Integration with Language Features

#### 4.6.1 Function Calls
- Arguments can be passed by value (move/copy)
- Arguments can be passed by reference (`&T`/`&mut T`)
- Return values can be moved or referenced

Example:
```ferra
fn process(s: &String) -> Int {
    s.len()  // Can read through &T
}

let s = String::from("hello");
let len = process(&s);  // Pass by reference
println(s);            // OK: s is still owned
```

#### 4.6.2 Data Structures
- Ownership rules apply recursively
- Fields can be borrowed independently
- Drop order follows field declaration order

Example:
```ferra
data User {
    name: String,
    age: Int
}

let user = User {
    name: String::from("Alice"),
    age: 30
};

let name_ref = &user.name;  // Borrow just the name field
println(*name_ref);         // OK: Can read name
```

#### 4.6.3 AI Integration
- `ai::assume(nll="noalias")` for borrow verification
- AI-assisted error explanations
- Pattern recognition for common ownership issues
- Automatic refactoring suggestions
- Example:
  ```ferra
  // Basic AI verification
  %2<i32> = ai.assume(nll="noalias") %1  // AI tag for borrow verifier

  // Advanced AI integration
  fn process(data: &mut [T]) {
      ai::assume(nll="noalias", bounds="checked") {
          // AI verifies no aliasing and bounds safety
          for item in data {
              process_item(item);
          }
      }
  }

  // AI-assisted refactoring
  ai::refactor::<"ownership"> {
      // AI suggests ownership improvements
      let result = complex_operation();
      // AI might suggest:
      // - Converting to reference
      // - Using move semantics
      // - Implementing Clone
  }
  ```

#### 4.6.4 Type Inference Integration
The borrow checker integrates with Ferra's Hindley-Milner type inference system:

```ferra
// Type inference with borrowing
fn process<T>(data: &[T]) -> &T {
    &data[0]  // Compiler infers lifetime of returned reference
}

// Row polymorphism with borrowing
fn update_fields<T: { name: String, age: Int }>(record: &mut T) {
    record.name = String::from("new name");  // Mutably borrows name field
    record.age += 1;                        // Mutably borrows age field
}
```

See `DESIGN_TYPE_INFERENCE.md` for details on:
- Bidirectional type inference
- Row polymorphism rules
- Lifetime inference algorithm

## 5. Drawbacks
- Learning curve for ownership concepts
- Some patterns require restructuring
- Limited to single-threaded in v0.1
- No explicit lifetime annotations yet

## 6. Security & Privacy
- Prevents memory safety vulnerabilities
- No undefined behavior from memory errors
- Clear ownership boundaries
- Safe FFI through extern blocks

## 7. Implementation Plan
- **Phase 1 (Q3 2025)**
  - Basic ownership rules
  - Move semantics
  - Copy semantics
  - Simple borrow checking
  - Core type system integration
  - Basic error reporting

- **Phase 1.5 (Q3 2025)**
  - Borrow checker UX implementation
  - Error message templates
  - IDE integration hooks
  - Test suite for error cases
  - Performance benchmarking
  - Common pattern detection
  - Interactive tutorials
  - Documentation generation
  - Example cookbook
  - Migration tools

- **Phase 2 (Q4 2025)**
  - Enhanced borrow checker
  - Lifetime inference
  - Positive-first error messages
  - Integration with concurrency
  - Advanced pattern matching
  - FFI safety guarantees
  - Cross-language interop
  - Tooling ecosystem

## 8. Migration Strategy
- New language; no migration needed
- Clear documentation and examples
- IDE support for ownership hints
- Interactive tutorials
- Common pattern cookbook
- Migration guide for Rust developers
- Gradual introduction of features
- Learning path:
  1. Basic ownership (Day 1)
  2. Borrowing and references (Week 1)
  3. Advanced patterns (Month 1)
  4. Expert usage (Quarter 1)
- Tooling support:
  - Ownership visualization
  - Pattern suggestions
  - Quick fixes
  - Refactoring tools
- Community resources:
  - Example repositories
  - Best practices guide
  - Common pitfalls
  - Performance tips
- Integration guides:
  - Rust interop
  - C/C++ FFI
  - Python bindings
  - WebAssembly

## 9. Unresolved Questions
### High Priority
- OB-1: Explicit lifetime annotations
- OB-2: Non-lexical lifetimes
- OB-3: Interior mutability
- OB-4: Async/await interaction
- OB-5: Unsafe code rules

### Medium Priority
- COPY-1: Copy trait definition
- COPY-2: Array copy semantics
- BORROW-1: Partial borrows
- BORROW-2: Pattern matching

### Low Priority
- BORROW-3: Custom operators
- BORROW-4: Advanced patterns
- BORROW-5: FFI integration

## 10. Future Possibilities
- Higher-rank lifetimes
- Associated types
- Custom operators
- Advanced pattern matching
- Type-level programming

## 11. References
1. [RFC-001: Syntax and Grammar](./RFC-001_SYNTAX_GRAMMAR.md)
2. [RFC-002: Core Semantics](./RFC-002_CORE_SEMANTICS.md)
3. [RFC-004: Concurrency Model](./RFC-004_CONCURRENCY_MODEL.md)
4. [OWNERSHIP_MODEL_PRINCIPLES_V0.1.md](../OWNERSHIP_MODEL_PRINCIPLES_V0.1.md)
5. [OWNERSHIP_BORROW_CHECKER.md](../OWNERSHIP_BORROW_CHECKER.md)
6. [CORE_SEMANTICS_V0.1.md](../CORE_SEMANTICS_V0.1.md)
7. [DESIGN_TYPE_INFERENCE.md](../DESIGN_TYPE_INFERENCE.md)
8. [DESIGN_DIAGNOSTICS.md](../DESIGN_DIAGNOSTICS.md)
9. [FFI_C_CPP.md](../FFI_C_CPP.md)
10. [diagnostic_codes.md](../diagnostic_codes.md) 