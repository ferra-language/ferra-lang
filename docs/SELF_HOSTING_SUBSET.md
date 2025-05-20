# Ferra Self-Hosting Subset Specification v0.1

> **Status:** Initial Draft - Module 1.8 · Step 1.8.1

## 1. Introduction and Goals

This document defines the **Ferra Self-Hosting Subset v0.1**. This subset comprises the minimal set of Ferra language features and standard library APIs deemed sufficient for writing initial parts of the Ferra compiler, its associated tools (e.g., `lang new`), or other command-line utilities for the Ferra ecosystem.

**Goals:**

*   Enable early development of Ferra tooling in Ferra itself.
*   Provide a focused target for the initial compiler implementation.
*   Validate the core language design for practical systems programming tasks.

**Non-Goals:**

*   This subset is not intended to be the full Ferra v0.1 language specification.
*   It is not initially targeted at general-purpose application development beyond compiler/tooling.

## 2. Guiding Principles for Subset Selection

*   **Core Utility**: Prioritize features essential for parsing text, manipulating data structures (like ASTs), basic I/O, and implementing command-line interfaces.
*   **Simplicity of Implementation**: Favor features that have a relatively straightforward implementation path in the initial bootstrap compiler.
*   **Alignment with v0.1 Specs**: Ensure the subset is a strict subset of features already defined in `SYNTAX_GRAMMAR_V0.1.md`, `CORE_SEMANTICS_V0.1.md`, and `STDLIB_CORE_V0.1.md`.
*   **Foundation for Iteration**: The subset should be a stable base that can be expanded as the Ferra compiler matures.

## 3. Core Language Features Included

This subset includes the following core language features, as defined in `docs/SYNTAX_GRAMMAR_V0.1.md` and `docs/CORE_SEMANTICS_V0.1.md`:

### 3.1. Lexical Structure

*   Comments (line `//` and block `/* ... */`).
*   Keywords (e.g., `fn`, `let`, `var`, `if`, `else`, `while`, `for`, `in`, `match`, `return`, `true`, `false`, `data`).
*   Identifiers (Unicode based).
*   Literals: Integers (decimal, hex, binary, octal), Floats (basic forms), Strings (with common escapes), Booleans.
*   Operators: Arithmetic (`+`, `-`, `*`, `/`, `%`), Comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`), Logical (`&&`/`and`, `||`/`or`, `!`), Assignment (`=`, `+=`, etc.), Range (`..`, `..=`).
*   Punctuation: `()`, `{}`, `[]`, `,`, `:`, `->`, `=>`, `?` (for error propagation).

### 3.2. Declarations

*   Function definitions: `fn identifier(param: Type, ...) -> ReturnType { ... }`
*   Variable bindings: `let identifier: Type = expression;` and `var identifier: Type = expression;` (type annotation optional if inferable for `let`).
*   Data structures: `data Identifier { field1: Type, field2: Type, ... }` (simple product types/structs).
*   Data structures (enums with data): `data EnumName { Variant1(Type1, Type2), Variant2 { field: Type }, Variant3 }`

### 3.3. Statements & Expressions

*   Expression statements.
*   Blocks: `{ Statement* }` or indented blocks.
*   Statement termination: Newline-sensitive and explicit semicolon `;`.
*   Control Flow:
    *   `if condition Block (else (Block | IfStatement))?` (as statement).
    *   `if condition Block else Block` (as expression).
    *   `while condition Block`
    *   `for identifier in expression Block`
    *   `match expression { Pattern => Expression, ... }`
    *   `return expression?`
    *   `break` and `continue` (label-less).
*   Function calls: `identifier(arg1, arg2)`.
*   Member access for data structures: `instance.field`.
*   Error propagation: `expression?`.

### 3.4. Type System Forms (from `docs/SYNTAX_GRAMMAR_V0.1.md`)

*   Primitive types implied by literals (e.g., `Int`, `Float`, `Bool`, `String`, `Char`).
*   Char literals use single quotes, e.g., `'A'`, and represent a single Unicode scalar value.
*   Named types (identifiers, qualified identifiers if module system is basic).
*   Tuple types: `(Type1, Type2, ...)`, `()` (unit type).
*   Array/Vector type: `[Type]` (dynamic, corresponds to `Vector<T>`).
*   Function types: `fn(Type1, ...) -> ReturnType`.
*   Generic types for standard library usage: `Identifier<TypeParam1, ...>` (e.g., `Result<T, E>`, `Vector<T>`).

### 3.5. Error Handling

*   Usage of `Result<T, E>` (as defined in `STDLIB_CORE_V0.1.md`).
*   `match` expressions for unwrapping `Result` values.
*   The `?` operator for error propagation.

## 4. Standard Library APIs Included

The following APIs from `docs/STDLIB_CORE_V0.1.md` are essential for the self-hosting subset:

### 4.1. Core (`core::*`)

*   `Result<T, E>`: `Ok(T)`, `Err(E)`, methods like `is_ok()`, `is_err()`, `unwrap()`, `expect()`.
*   `Option<T>`: `Some(T)`, `None`, and associated methods (if deemed critical for initial subset, otherwise Result can cover many cases).
*   `Error` (or a specific error type like `IOError` from `io::*` for I/O, and a general `ParseError` for compiler tasks - to be precisely defined for self-hosting).

### 4.2. I/O (`io::*`)

*   `print(message: String)`
*   `println(message: String)`
*   `eprintln(message: String)`
*   `read_line() -> Result<String, IOError>`
*   File Operations:
    *   `fs::read_to_string(path: String) -> Result<String, IOError>`
    *   (Potentially `fs::write(path: String, content: String) -> Result<(), IOError>` if tools need to write files).
*   `IOError` data type for error handling.

### 4.3. Collections (`collections::*`)

*   `Vector<T>`:
    *   `Vector::new() -> Vector<T>`
    *   `vector.push(value: T)`
    *   `vector.pop() -> Option<T>`
    *   `vector.get(index: Int) -> Option<&T>`
    *   `vector.len() -> Int`
    *   `vector.is_empty() -> Bool`
    *   Iteration support for `for item in vector`.
*   `Map<K, V>` (with `K` as `String` or `Int` for v0.1):
    *   For this subset, `K` is restricted to `Int` or `String` as a full trait system for key constraints is deferred.
    *   `Map::new() -> Map<K, V>`
    *   `map.insert(key: K, value: V) -> Option<V>`
    *   `map.get(key: K) -> Option<&V>`
    *   `map.remove(key: K) -> Option<V>`
    *   `map.contains_key(key: K) -> Bool`
    *   `map.len() -> Int`

### 4.4. String (`string::*`)

*   String type itself with basic operations implied by literals and `+` for concatenation.
*   `String::from_int(value: Int) -> String`
*   `string.len() -> Int`
*   `string.is_empty() -> Bool`
*   `string.chars() -> Vector<Char>` (Returns a `Vector<Char>` for this subset; a more general iterator protocol is deferred).
*   `string.split(separator: String) -> Vector<String>`
*   `string.trim() -> String`

### 4.5. Character (`char::*`)

*   `Char` type.
*   Basic properties if needed (e.g., `is_whitespace()`, `is_digit()`).

## 5. Ownership & Memory Model

*   The Ferra ownership and borrowing model, as outlined in `docs/OWNERSHIP_MODEL_PRINCIPLES_V0.1.md`, applies to this subset.
*   Focus will be on patterns that are common in compiler construction (e.g., tree traversal, data transformation) and can be expressed safely.
*   For the initial self-hosting efforts, emphasis will be on single-threaded execution, deferring complex concurrent memory patterns.

## 6. Basic Modularity

*   For v0.1 self-hosting, we assume a simple module system where each `.ferra` file can be a module.
*   An import mechanism will be required. Example placeholder: `import my_project::utilities;` or `import relative_path::module_name;` (Exact syntax TBD under SELFHOST-SUBSET-1).
*   Visibility markers (e.g., `pub`) for items intended to be used by other modules.

## 7. Excluded Features (Notable v0.1 Aspects NOT in this Initial Subset)

*   Advanced concurrency features (async/await, actors) beyond single-threaded execution.
*   Full Foreign Function Interface (FFI).
*   Macros (beyond any built-in ones like `println!`, if `println` is a macro).
*   Trait definitions and implementations by users (beyond what's implicitly used by built-in types or core stdlib like `for..in` over `Vector`).
*   Advanced generic programming features beyond what's needed for stdlib collections/Result.
*   UI-DSL features.
*   Explicit memory unsafe operations (`unsafe` keyword, raw pointers), unless absolutely unavoidable for specific low-level bootstrapping tasks and heavily controlled.

## 8. `Ferra.toml` Manifest for Self-Hosting Tools

A tool or compiler component written in this Ferra subset would have a `Ferra.toml` like:

```toml
[package]
name = "ferra_lexer_tool"
version = "0.1.0"

[dependencies]
# Minimal or no external Ferra dependencies for initial self-hosting tools
# std = "0.1.0" # Implicitly available or specified if stdlib is versioned
```

## 9. Target Use Cases for Self-Hosting v0.1

*   **Lexer/Tokenizer**: Reading Ferra source code and producing a stream of tokens.
*   **Parser Utilities**: Components of a recursive descent parser.
*   **AST Pretty-Printer**: For debugging the parser and AST structures.
*   **Basic CLI Tools**: Such as the `lang new` project scaffolder itself, or a simple test runner.
*   **Build Script Components**: Simple scripts for managing the Ferra compiler build process.

## 10. Open Questions

| Tag                | Issue                                                                                                | Status   |
|--------------------|------------------------------------------------------------------------------------------------------|----------|
| SELFHOST-SUBSET-1  | Finalize the exact syntax and semantics for the minimal module system (imports/exports).               | Open     |
| SELFHOST-SUBSET-2  | Define `Option<T>` explicitly in `STDLIB_CORE_V0.1.md` and include fully in subset if deemed critical. | Open     |
| SELFHOST-SUBSET-3  | Iterator pattern for collections: should a basic form be included for `for` loops over `Vector`?     | Open     |
| SELFHOST-SUBSET-4  | Minimum diagnostic features required from the bootstrap compiler itself (e.g., colorized output, structured error messages)? | Open     |

---
This document will evolve as the first parts of the Ferra compiler are implemented in this subset. 