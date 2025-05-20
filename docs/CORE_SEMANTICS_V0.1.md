# Ferra Core Semantics v0.1

This document outlines the core semantics for Ferra v0.1, defining the behavior and meaning of language constructs. It is intended to be read alongside the `docs/SYNTAX_GRAMMAR_V0.1.md`.

## 1. Introduction

*   **Scope**: This document describes the fundamental runtime behavior, type system basics, and evaluation rules for Ferra v0.1, corresponding to the syntax defined in `SYNTAX_GRAMMAR_V0.1.md`.
*   **Versioning**: This is version 0.1 of the core semantics.
*   **Out-of-Scope for v0.1 Semantics (to be detailed in future documents/RFCs)**:
    *   Detailed ownership, borrowing, and lifetime rules (see `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md` for initial thoughts).
    *   Concurrency model (async/await runtime behavior, actor model specifics).
    *   Standard library (`std`) content, types, and their detailed semantics (e.g., `Result<T, E>`, `Option<T>`).
    *   Metaprogramming and macro expansion semantics.
    *   Foreign Function Interface (FFI) details.

## 2. Execution Model (Bird's-Eye View)

*   **Entry Point**: A Ferra executable program begins execution by calling a function named `main` defined in the root module. This `main` function must have the signature `fn main() -> Int` (or `fn main() -> ()` if exit codes aren't critical for v0.1). Its integer return value becomes the process exit code (0 typically indicating success).
*   **Threading**: For v0.1, execution is conceptually single-threaded and follows a deterministic order.
*   **Evaluation Strategy**: Ferra is primarily an eagerly evaluated language.
*   **Expression-Oriented**: Many constructs, including `if` and `match`, are expressions and yield a value. Statements that do not naturally yield a value (like `while` loops or `var` assignments) implicitly produce the `Unit` type (`()`).

## 3. Built-in Types

This section lists the canonical built-in types for Ferra v0.1 and their key invariants.

### 3.1. Scalar Types

*   **`Bool`**: Represents boolean values.
    *   Literals: `true`, `false`.
*   **`Int`**: Represents a signed integer.
    *   Default Size: 64-bit (two's complement).
    *   Literals: As defined in `SYNTAX_GRAMMAR_V0.1.md` (e.g., `10`, `0xFF`, `0o77`, `0b1010`). Underscores (`_`) can be used as visual separators in numeric literals (e.g., `1_000_000`, `0xFF_EC_DE`), and are ignored by the lexer for value determination.
    *   ⚠️ **TBD**: Precise rules for numeric conversions, overflow behavior (e.g., wrapping, trapping in debug, saturating).
*   **`Float`**: Represents a floating-point number.
    *   Default Size: 64-bit (IEEE 754 double-precision).
    *   Literals: As defined in `SYNTAX_GRAMMAR_V0.1.md` (e.g., `3.14`, `1.0e-5`). Underscores (`_`) can be used as visual separators (e.g., `1_000.000_001`), and are ignored by the lexer for value determination.
    *   ⚠️ **TBD**: Support for hexadecimal or binary float literals.
*   **`Char`**: Represents a single Unicode scalar value.
    *   Default Size: 32-bit (representing a Unicode codepoint).
    *   Literals: As defined in `SYNTAX_GRAMMAR_V0.1.md` (e.g., `'a'`, `'\n'`, `'\u{1F600}'`).
*   **`String`**: Represents a sequence of Unicode characters, encoded as UTF-8.
    *   Immutability: Strings are immutable by default.
    *   Behavior: Conceptually similar to Rust's `String` or Swift's `String` (heap-allocated, growable text).
    *   Slicing: String slicing operations will be available (details TBD with std-lib/operator overloading).
*   **`Unit`**: Represents the absence of a specific value.
    *   Literal: `()`.
    *   Usage: Automatically returned by functions with no explicit `return` expression, and by expressions/statements that don't produce another value (e.g., `var` assignments, loops).

### 3.2. Compound Types (Built-in structural forms)

*   **Tuple Types**: `(T1, T2, ... TN)`
    *   Representation: An ordered, fixed-size collection of heterogeneous values.
    *   Value Semantics: Tuples are value types; assignment copies the tuple's contents.
    *   Examples: `()`, `(Int, String)`, `(Float, Float, Bool)`.
*   **Array Types**: `[T]`
    *   Representation: A dynamically-sized, ordered collection of homogeneous values `T`.
    *   Allocation: Typically heap-allocated and growable.
    *   Mutability: Governed by `let`/`var` binding of the array instance and potentially methods if `T` allows mutation through shared references (details tied to ownership).
    *   Examples: `[Int]`, `[String]`, `[(Int, Bool)]`.
*   **Function Types**: `fn(T1, T2, ...) -> R`
    *   Representation: The type of a function, capturing parameter types and return type.
    *   Usage: Allows functions to be passed as arguments, returned from other functions, and assigned to variables.
*   **`data` Types (User-Defined Records/Structs)**:
    *   Instances of `data` classes (defined with `data MyData { field: Type, ... }`) are compound types.
    *   **Structural Compatibility (Row Polymorphism)**: Due to row polymorphism (see `DESIGN_TYPE_INFERENCE.md` and `FRONTEND_ENHANCEMENTS.MD`), functions may accept instances of different `data` types if they structurally match the required fields for a given operation. For example, a function expecting a record with a `name: String` field can accept any `data` instance that provides such a field, regardless of other fields it may contain. This allows for a degree of structural subtyping.
*   **Reserved Generic Type Names (Semantics Deferred to Standard Library)**:
    *   `Result<T, E>`: Intended for error handling (representing success `T` or error `E`).
    *   `Option<T>`: Intended for representing optional values (present `T` or absent).
    *   While the *syntax* for `GenericType` (e.g., `Result<Response>`) is parsed, the specific semantics, methods, and usage patterns for `Result` and `Option` will be defined as part of the standard library specification.

## 4. Bindings & Mutability

Bindings associate a name (identifier) with a value or a memory location.

*   **`let` Bindings (Immutable)**:
    *   Syntax: `let name: Type = value;` (see `SYNTAX_GRAMMAR_V0.1.md`, §2.1).
    *   Semantics: Creates an immutable binding. The name `name` cannot be reassigned to a different value or memory location after initialization.
    *   If the bound value is itself mutable (e.g., a `var`-bound struct instance referred to by an immutable `let` reference, or an array with mutable elements), internal mutation might still be possible depending on ownership and borrowing rules (⚠️ **TBD** with ownership RFC).
*   **`var` Bindings (Mutable)**:
    *   Syntax: `var name: Type = value;` (see `SYNTAX_GRAMMAR_V0.1.md`, §2.1).
    *   Semantics: Creates a mutable binding. The name `name` can be reassigned to a different value of the same type.
    *   The expression `name = new_value` (assignment to a `var`) evaluates to `Unit` (`()`).
*   **Shadowing**:
    *   Allowed: A new binding (`let` or `var`) can shadow a previous binding of the same name in an inner scope.
    *   Not Allowed in Same Scope: Re-declaring a name with `let` or `var` in the exact same block scope where it's already defined is an error. (e.g., `let x = 1; let x = 2; // ERROR in same block`).
      ```ferra
      let a = 10
      {
          let a = "hello" // OK: a (String) shadows outer a (Int)
          // var a = true // ERROR: cannot redefine a in this inner scope
      }
      // Here, a is still 10 (Int)
      ```

## 5. Evaluation & Side-Effect Rules

*   **Order of Evaluation of Sub-expressions**: Sub-expressions are generally evaluated left-to-right.
    *   Example: In `foo() + bar()`, `foo()` is fully evaluated before `bar()` is evaluated.
*   **Function Arguments**: All arguments to a function call are fully evaluated (left-to-right) before the function itself is called.
*   **Short-circuiting**: Logical operators `&&` (and `and`) and `||` (and `or`) exhibit short-circuiting behavior:
    *   For `a && b`, `b` is only evaluated if `a` evaluates to `true`.
    *   For `a || b`, `b` is only evaluated if `a` evaluates to `false`.
*   **Sequence Points**: Explicit sequence points occur at semicolons (`;`) or statement-terminating newlines (as defined in `SYNTAX_GRAMMAR_V0.1.md`, §1.1). All side effects of a preceding expression/statement are complete before the next expression/statement begins evaluation.
    *   A code formatter (`ferrafmt`) will be responsible for normalizing code to a canonical style regarding optional semicolons and newline usage, adhering to the language grammar.
*   **Laziness**: Ferra v0.1 is primarily eagerly evaluated. Explicit lazy evaluation constructs are not part of the core v0.1 semantics (⚠️ **TBD** for future RFCs if demand arises for iterators, generators, etc.).

## 6. Control-Flow Semantics

Refers to syntax in `SYNTAX_GRAMMAR_V0.1.md` (§4 and §5).

*   **`if`/`else` Expressions & Statements**:
    *   As an expression (e.g., `let x = if cond { val_a } else { val_b }`):
        *   The `cond` expression is evaluated first.
        *   If `cond` is `true`, the `val_a` block/expression is evaluated, and its result becomes the value of the `if` expression.
        *   If `cond` is `false`, the `val_b` (else) block/expression is evaluated, and its result becomes the value of the `if` expression.
        *   The `else` branch is mandatory for an `if` expression.
        *   The types of the `true` branch and the `false` branch **MUST** unify to a common supertype (or be identical). This common type is the type of the `if` expression.
    *   As a statement (e.g., `if cond { do_this() } else { do_that() }`):
        *   Evaluation follows the same logic.
        *   If used as a statement, the resulting value (if any) is discarded, and the overall `if` statement effectively produces `Unit`.
        *   The `else` branch is optional for an `if` statement.
*   **`while` Loops**: `while cond Block`
    *   The `cond` expression is evaluated.
    *   If `true`, the `Block` is executed. Then, `cond` is re-evaluated.
    *   This continues until `cond` evaluates to `false`.
    *   A `while` loop expression itself produces `Unit` (`()`).
*   **`for...in` Loops**: `for item in iterable Block`
    *   The `iterable` expression is evaluated to produce an iterator (details of iterator protocol ⚠️ **TBD** with std-lib/trait design).
    *   The `Block` is executed for each `item` produced by the iterator.
    *   A `for` loop expression itself produces `Unit` (`()`).
*   **`break` Statement**: `break`
    *   Immediately exits the innermost enclosing `while` or `for` loop.
    *   A `break` statement itself produces `Unit` (`()`). (Note: `break value` is ⚠️ **TBD** via future RFC).
*   **`continue` Statement**: `continue`
    *   Immediately skips to the next iteration of the innermost enclosing `while` or `for` loop. For `while`, this means re-evaluating the condition. For `for`, this means getting the next item from the iterator.
    *   A `continue` statement itself produces `Unit` (`()`).
*   **`return` Statement**: `return expr?`
    *   Immediately exits the current function.
    *   If `expr` is provided, its value is returned by the function.
    *   If no `expr` is provided, `Unit` (`()`) is returned. This is equivalent to `return ()`.

## 7. Scope & Lifetime Basics (Preliminary)

*   **Lexical Scoping**: Ferra uses lexical (static) scoping. The scope of a binding is determined by the structure of the source code.
*   **Blocks and Scope**: Each block delimited by `{...}` or an `IndentedBlock` introduces a new lexical scope.
    *   Bindings declared within a scope are generally not visible outside that scope.
*   **Value Lifetimes (Conceptual Overview - Details in Ownership RFC)**:
    *   Values exist for a certain duration (their lifetime).
    *   For v0.1, without a full ownership/borrowing system formally defined, we assume a general model where values managed by the system (e.g., heap allocations for strings, arrays) live as long as they are reachable or referenced.
    *   The precise rules for when values are dropped/deallocated, and how references interact with lifetimes, are critical and will be detailed in the Ownership Model RFC. This section is a placeholder for that more detailed discussion.

## 8. Error Handling Outline (Preliminary)

This section provides a forward pointer to the intended error handling mechanisms, particularly the postfix `?` operator.

*   **Postfix `?` Operator**: (Syntax in `SYNTAX_GRAMMAR_V0.1.md`, §4)
    *   Intended Use: To propagate errors from functions or operations that return a `Result<T, E>`-like type (where `Result` is expected to be part of the standard library).
    *   High-Level Semantics:
        *   If `expr?` is applied to a value representing success (e.g., `Ok(value)` or equivalent), it unwraps the success value (e.g., `value`).
        *   If `expr?` is applied to a value representing an error (e.g., `Err(error_value)` or equivalent), it causes an early exit from the current function, propagating the `error_value` (after potential conversion to the function's declared error type).
    *   Parser & Type Checker: The parser must recognize `?` as a postfix operator. The type checker must ensure it's applied to a type that supports this propagation pattern (e.g., a future `Result` or `Option` type from the std-lib).
    *   ⚠️ **TBD**: The exact mechanics, including the trait or interface that `?` operates on, and error type conversion rules, will be defined in conjunction with the standard library's `Result` and `Option` types and potentially an error handling RFC.

## 9. Open Questions & Future RFCs (Semantic Focus)

This list captures semantic topics requiring further design or deferred to future RFCs.

*   Detailed semantics for `Result<T, E>` and `Option<T>` (standard library).
*   Precise numeric conversion rules (e.g., `Int` to `Float`, between different sized integers if introduced).
*   Integer overflow behavior (wrapping, trapping, saturating).
*   Iterator protocol for `for...in` loops.
*   Support for hexadecimal or binary float literals.
*   Sized integer types (e.g., `i8`, `u32`, `i128`, `u128`).
*   Fixed-size array semantics (beyond `[T]`).
*   Detailed pointer/reference types and comprehensive ownership & borrowing semantics.
*   Concurrency and asynchronous memory model (details for `async`/`await` beyond syntax).
*   Metaprogramming: Macro expansion rules, hygiene.
*   Module system: Namespace resolution, import/export semantics, visibility rules.
*   FFI calling conventions and data marshalling.
*   Dynamic dispatch (if traits/interfaces lead to it).
*   Reflection capabilities (if any).
*   Details of `break value` from loops.

---

## Appendix A: Glossary

*   **Binding**: An association between a name (identifier) and a value or a memory location. In Ferra, created with `let` (immutable) or `var` (mutable).
*   **Expression-Oriented Language**: A language where most constructs are expressions that evaluate to a value. For example, in Ferra, `if` and `match` can be used as expressions.
*   **Immutability**: A property of data that cannot be changed after it is created. `let` bindings create immutable references to values. Some types (like `String`) are inherently immutable.
*   **Mutability**: A property of data that can be changed after it is created. `var` bindings allow reassignment. Some data structures might allow internal mutation.
*   **Lexical Scoping**: A scoping mechanism where the scope of a name is determined by its location in the source code text (i.e., where it was declared).
*   **Shadowing**: The practice of declaring a new variable with the same name as a variable in an outer scope. The inner variable "shadows" (hides) the outer one within its scope.
*   **Side Effect**: An operation that modifies some state outside of its local environment, such as modifying a mutable variable, performing I/O, or calling a function that has side effects.
*   **Statement**: A unit of execution that performs some action. Unlike expressions, statements do not necessarily produce a value (or may produce `Unit` by default).
*   **Type Unification**: In the context of an `if`/`else` expression, the process by which the compiler determines a common, compatible type for the values produced by both branches.
*   **Unit Type (`()`)**: A type that has only one value, also written `()`. It is used to represent the absence of a specific value, often as the return type of functions that perform actions but don't return data, or as the result of expressions that are executed for their side effects.
*   **Value Semantics**: A property of types where variables directly hold the value, and assignment or passing as an argument typically involves copying the value. For example, tuple types in Ferra have value semantics.
