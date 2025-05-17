# Ferra Ownership & Borrowing Model: v0.1 Principles

This document outlines the foundational principles for Ferra's ownership and borrowing system. The primary goal is to achieve compile-time memory safety comparable to Rust, eliminating common errors like dangling pointers, data races, and use-after-free, without relying on a garbage collector. A secondary goal is to make this system as approachable as possible, aligning with Ferra's "Python-ease" aspiration, particularly through "positive-first" error messaging.

This is a v0.1 principles document and will serve as the basis for a more detailed RFC and specification.

## 1. Introduction & Goals

*   **Memory Safety**: Ferra aims for automatic memory management via compile-time checks, preventing memory-related bugs by default.
*   **Performance**: By avoiding a garbage collector, Ferra targets predictable performance characteristics suitable for systems programming, game development, and other performance-sensitive domains.
*   **Developer Experience**: While memory safety rules can be complex, Ferra will strive for:
    *   Clear, understandable rules.
    *   "Positive-first" error messages from the borrow checker, guiding users towards valid code rather than just pointing out errors (inspired by Dr. Hoare's feedback in `docs/Steps.md`).
    *   Sensible defaults and type inference to reduce annotation burden where possible.
*   **Inspiration**: The model draws heavily from Rust's successful ownership and borrowing system, adapting concepts to fit Ferra's overall design philosophy.
*   **Interoperability (FFI)**: Principles for interacting with code written in other languages (e.g., C ABI compatibility, data marshalling for FFI) are critical for a general-purpose language but will be detailed in a future RFC (⚠️ **TBD (FFI-1)**).

## 2. Core Principles

### 2.1. Ownership

*   **Single Owner**: Each value in Ferra has a variable that is its *owner*.
*   **Scope-Bound Lifetime**: A value is valid as long as its owner remains in scope. When the owner goes out of scope, the value is automatically *dropped* (i.e., its resources are deallocated).
    *   Example: `let s = String::from("hello"); // s owns the string data. When s goes out of scope, the string is dropped.`
*   **One Owner at a Time**: There can only be one owner of a particular piece of data at any given time.

### 2.2. Move Semantics (Default for Owning Types)

*   When an owning value is assigned to another variable, or passed as an argument to a function, ownership is *moved*.
*   The original variable is no longer considered valid and cannot be used (to prevent double-free errors).
    *   Example: `let s1 = String::from("hello"); let s2 = s1; // s1 is moved to s2. s1 cannot be used hereafter.`
*   This applies by default to types that manage resources on the heap (e.g., `String`, `[T]`, user-defined `data` types that are not `Copy`).

### 2.3. Copy Semantics (for `Copy` Types)

*   Types that have the `Copy` trait/marker are copied by value during assignment or function calls. The original variable remains valid and usable.
*   **Default `Copy` Types**: Built-in scalar types (`Int`, `Float`, `Bool`, `Unit`) are `Copy` by default. Tuples and arrays composed entirely of `Copy` types are also `Copy` by default.
    *   Example: `let x: Int = 5; let y = x; // y is a copy of x. Both x and y are valid.`
    *   Example: `let p1: (Int, Bool) = (10, true); let p2 = p1; // p2 is a copy of p1.`
*   Ferra will provide a way to mark simple `data` types as `Copy` if all their fields are also `Copy` (⚠️ **TBD (COPY-1)** Formal `Copy` trait definition).

    (* **Summary: `Copy` vs. `Move` for Built-in/Structural Types (v0.1)**
       | Type Category        | Behavior    | Notes                                      |
       |----------------------|-------------|--------------------------------------------|
       | `Int`, `Float`, `Bool`, `Unit` | `Copy`      | Simple scalar values.                      |
       | Tuples of `Copy` types| `Copy`      | e.g., `(Int, Bool)`                        |
       | Arrays of `Copy` types| `Copy`      | e.g., `[Int]` (if `Int` is `Copy`, array is `Copy` if passed by value, shallow copy of array structure, deep copy of elements) - Semantics for array copy needs care, deep vs shallow. For now, assume `[T]` when `T` is `Copy` implies elements are copied. True heap-allocated array *structure* might be `Move` by default. ⚠️ **TBD (COPY-2)** Array copy/move details. |
       | `String`             | `Move`      | Manages heap resource.                     |
       | `[T]` (general case) | `Move`      | Dynamic, heap-allocated list structure.    |
       | `data User {...}`    | `Move`      | By default, unless marked `Copy`.          |
       | `&T`, `&mut T`       | `Copy`      | References themselves are copied (like pointers). The data they point to is not, and borrow rules apply. |
    *)

### 2.4. Borrowing & References

To access data without taking ownership, Ferra uses *references* (borrows).

*   **Shared (Immutable) References (`&T`)**:
    *   Allows reading the data but not modifying it.
    *   Multiple shared references to the same data can exist simultaneously.
    *   Syntax (conceptual): `let r1 = &value; let r2 = &value;`
*   **Exclusive (Mutable) References (`&mut T`)**:
    *   Allows reading and modifying the data.
    *   If an exclusive reference exists to data, no other references (shared or exclusive) to that data can exist simultaneously within the same scope or overlapping lifetimes.
    *   This rule is key to preventing data races at compile time.
    *   Syntax (conceptual): `let r_mut = &mut value;`
*   **Reference Validity (Lifetimes - Conceptual)**:
    *   The compiler (via the borrow checker) ensures that references never outlive the data they point to (i.e., no dangling references).
    *   While explicit lifetime annotations (like Rust's `'a`) will be a more advanced topic (⚠️ **TBD** for a later RFC), the principle of lifetime validation by the compiler is core from v0.1.
    *   The compiler will infer lifetimes in many common cases.
    *   *(Example of a borrow check error illustrating positive-first messaging is in Section 4.)*

## 3. Impact on Language Constructs

*   **Function Calls**:
    *   Arguments can be passed by value (moving ownership for owning types, copying for `Copy` types).
    *   Arguments can be passed by shared reference (`&T`).
    *   Arguments can be passed by exclusive reference (`&mut T`).
*   **Assignments**:
    *   `let y = x;`: Moves or copies `x` to `y` based on `x`'s type.
    *   `var_mut = new_val;`: Moves or copies `new_val` into the memory location of `var_mut`.
*   **Data Structures (`data`, `[T]`, Tuples)**:
    *   Ownership rules apply to the data structure instance itself and recursively to its fields/elements if they are owning types.
    *   Accessing fields or elements will involve borrowing rules.
*   (Cross-reference to `CORE_SEMANTICS_V0.1.md` §4 for general binding/mutability rules can be added here if more detail is needed beyond what's in §5 below.)

## 4. "Positive-First" Error Messaging for the Borrow Checker

Inspired by Dr. Hoare's feedback in `docs/Steps.md`, Ferra's borrow checker will prioritize guiding the user, not just stating errors.

*   **Focus on Validity**: Messages should first explain what *is* valid or for how long a borrow *is* valid.
*   **Clear Contrast**: Clearly state the conflicting action and where it occurs.
*   **Actionable Hints**: Provide concrete, simple suggestions for resolving the error, such as restructuring code (e.g., scopes, `with` blocks if introduced), cloning data, or changing borrow types if appropriate.

    *Example Diagnostic (Conceptual) - Scenario 1: Mutable after Immutable Borrow*
    ```text
    error: cannot borrow `config` as mutable because it is also borrowed as immutable
      --> main.ferra:10:5
        |
    7   |     let reader = &config; // immutable borrow occurs here
        |                  ------- immutable borrow later used here (line 12)
    ... 
    10  |     config.update();    // mutable borrow occurs here
        |     ^^^^^^ mutable borrow occurs here

    ✔ `config` is immutably borrowed at line 7 and this borrow is valid until line 12.
    ✘ A mutable borrow of `config` starts at line 10, which conflicts with the active immutable borrow.

    Hint: To allow mutation, ensure all immutable borrows of `config` end before line 10.
          Alternatively, consider if `config.update()` can operate on an immutable `&config`
          or if `config` needs to be cloned before the immutable borrow.
    ```

    *Example Diagnostic (Conceptual) - Scenario 2: Two Mutable Borrows*
    ```ferra
    let mut data = ...
    let r1 = &mut data; // First mutable borrow
    let r2 = &mut data; // Second mutable borrow - ERROR!
    // use r1, r2
    ```
    *Conceptual Diagnostic Output:*
    ```text
    error: cannot borrow `data` as mutable more than once at a time
      --> main.ferra:3:14
        |
    2   |     let r1 = &mut data; // First mutable borrow
        |              ---------- first mutable borrow occurs here
    3   |     let r2 = &mut data; // Second mutable borrow - ERROR!
        |              ^^^^^^^^^^ second mutable borrow occurs here

    ✔ `data` is mutably borrowed by `r1` starting at line 2.
    ✘ Another mutable borrow of `data` by `r2` starts at line 3, while `r1`'s borrow is still active.

    Hint: A value can only have one mutable reference at a time. Ensure the first mutable borrow (`r1`)
          is no longer used before creating the second mutable borrow (`r2`), or restructure
          your code to use a single mutable reference if modifications need to happen sequentially.
    ```

## 5. Relationship to `let` and `var`

*   `let name = value;`: `name` takes ownership if `value` is an owning rvalue. If `value` is a reference, `name` is a reference with the same properties (mutability, lifetime) as `value`.
*   `var name = value;`: Similar to `let` for initial ownership/borrow. `name` can later be reassigned. Reassignment follows move/copy semantics for the new value being assigned.
*   The mutability of a binding (`var`) is distinct from the mutability of a reference (`&mut T`). A `let` binding can hold an `&mut T` if the reference itself is not reassigned. A `var` binding can hold an `&T` and later be reassigned to another `&T` or a different value.
*   For general rules on bindings, mutability, and shadowing, see `docs/CORE_SEMANTICS_V0.1.md`, Section 4.

## 6. Initial Simplifications & Deferrals for v0.1 Principles

To keep the initial model focused:

*   **Explicit Lifetime Annotations**: ⚠️ **TBD (OB-1)** Not part of v0.1 principles; lifetime inference will be relied upon. Advanced lifetime specification is a future RFC.
*   **Non-Lexical Lifetimes (NLL)**: ⚠️ **TBD (OB-2)** Assumed as a goal for the borrow checker implementation, but formal specification details are for a future RFC.
*   **Interior Mutability**: ⚠️ **TBD (OB-3)** Patterns like Rust's `Cell` or `RefCell` are deferred (likely std-lib or advanced language feature).
*   **`async`/`await` Interaction**: ⚠️ **TBD (OB-4)** Detailed rules for ownership with `async`/`await` are deferred for a future RFC.
*   **Unsafe Code**: ⚠️ **TBD (OB-5)** The existence and rules for an `unsafe` escape hatch are deferred for a future RFC.

## 7. Open Questions & Next Steps for Specification

*   Formal definition of the `Copy` trait/marker and rules for deriving it (see ⚠️ **TBD (COPY-1)**).
*   Detailed semantics for array copy vs. move behavior (see ⚠️ **TBD (COPY-2)**).
*   Rules for how borrowing interacts with indexing into arrays/slices.
*   Specifics of lifetime inference algorithms (related to ⚠️ **TBD (OB-1)**).
*   Detailed error message catalog for common borrow checking scenarios.
*   Semantic impact of ownership on pattern matching.
*   Drop order for fields within a struct/data type and for local variables within a scope (deterministic destructors).

This document sets the stage. The next step will be to elaborate these principles into a more formal specification, likely as part of a dedicated Ownership Model RFC, and then to implement and refine the borrow checker. 

---

## Appendix A: Glossary of Ownership & Borrowing Terms

*   **Owner**: The variable that is responsible for a piece of data. When the owner goes out of scope, the data is dropped.
*   **Borrow**: To create a reference to data without taking ownership of it. Borrows can be shared (immutable, `&T`) or exclusive (mutable, `&mut T`).
*   **Move**: The process by which ownership of data is transferred from one variable to another. After a move, the original variable is typically no longer valid.
*   **Copy**: The process of creating a new, independent instance of data. Both the original and the copied data are valid. Applies to types marked with the `Copy` trait.
*   **Lifetime**: The scope or duration for which a reference is valid. The borrow checker ensures references do not outlive the data they point to.
*   **Drop**: The process of deallocating resources held by a value when its owner goes out of scope (or it is explicitly dropped). 