# Ferra Type Inference Design v0.1

This document specifies the design for the Ferra v0.1 type inference system. It builds upon the language syntax defined in `docs/rfc/RFC-001_SYNTAX_GRAMMAR.md`, the core semantics in `docs/rfc/RFC-002_CORE_SEMANTICS.md`, and the AST structure in `AST_SPECIFICATION.md`.

## 1. Introduction & Goals

*   **Purpose**: To automatically deduce the types of expressions and declarations where they are not explicitly annotated, providing both safety and developer convenience.
*   **Algorithm Foundation**: The system will be based on a Hindley-Milner (HM) type inference algorithm, extended to support features like gradual typing and potentially row polymorphism.
*   **Strong Typing**: Despite inference, Ferra aims for a strong, static type system. Successful type inference will result in a fully typed program (or well-defined parts in a gradual typing context).
*   **Rust-Class Monomorphisation**: The type system and inference should ultimately support monomorphisation similar to Rust for performance. This means that generic code, after successful type checking and inference, is expected to be specialized (i.e., concrete versions generated for each specific type usage) during a later compiler phase (e.g., IR generation or code generation), ensuring that unspecialized generic code is not executed directly at runtime.
*   **Gradual Typing**: Support the `_` wildcard in type annotations to allow parts of the program to have types inferred while others are explicit.
*   **Developer Experience**: Prioritize clear error messages when type inference fails or conflicts arise. Leverage bidirectional inference to improve error quality and inference power.
*   **Scope Note (v0.1)**: For v0.1, the type system generally forbids implicit numeric widening or conversions (e.g., an `Int` cannot be automatically used where a `Float` is expected without an explicit cast or conversion function). This promotes explicitness but may be revisited for ergonomic improvements via RFCs.

## 2. Core Algorithm: Hindley-Milner (HM) Overview

*   **Type Variables**: Introduce fresh type variables (e.g., `α`, `β`) for expressions or parts of types whose types are unknown.
*   **Type Schemes (Polytypes)**: For `let`-bound (but not `var`-bound in v0.1) polymorphic functions or values, generalize their types with universal quantifiers (e.g., `∀α. α -> α` for the identity function). Instantiate these schemes with fresh type variables at call/use sites.
*   **Constraint Generation**: Traverse the AST, generating type constraints based on the structure of the code.
    *   Literals have known types (e.g., `10` is `Int`).
    *   Operators constrain their operands and result types (e.g., `+` on `Int`s produces `Int`).
    *   Function calls constrain argument types and return types based on the function's type signature.
    *   `let x = e;` constrains the type of `x` to be the type of `e`.
    *   `fn(x: T1) -> T2 { ... }` provides known types for `x` and the function's return.
*   **Unification**: Solve the collected constraints by finding a principal substitution (a mapping from type variables to types) that makes all constraints hold. Unification either succeeds with a substitution or fails if types are incompatible.
    *Example Unification Process:*
    ```
    Initial Constraints:
        type_of(x) = α
        type_of(10) = Int
        type_of(y) = β
        α = β         // From `let y = x;`
        α = Int       // From `x` being used in a context expecting `Int`

    Applying Substitutions:
    1. Substitute α with Int:  { α ↦ Int }
       Remaining Constraints:
           β = Int

    2. Substitute β with Int:  { β ↦ Int }
       No remaining constraints.

    Final Substitution: { α ↦ Int, β ↦ Int }
    ```
*   **Occurs Check**: A crucial part of unification to prevent infinite types (e.g., `α = List<α>`).
*   **Algorithm Complexity**: The core HM algorithm typically has near-linear time complexity in practice (often cited as `O(N log N)` or `O(N α(N))` where `α` is the inverse Ackermann function, effectively linear for practical program sizes).
*   **Polymorphism Scope (v0.1)**: The system will support **Rank-1 Polymorphism** (also known as shallow or predicative polymorphism). This means polymorphism is restricted to `let`-bindings; type variables quantified in a type scheme cannot themselves be instantiated with polymorphic types. Higher-rank polymorphism is deferred.

## 3. Application to Ferra's Type System (v0.1)

Type inference will apply to the following, as defined in `docs/rfc/RFC-001_SYNTAX_GRAMMAR.md` (§3) and `docs/rfc/RFC-002_CORE_SEMANTICS.md` (§3):

*   **Built-in Scalar Types**: `Int`, `Float`, `Bool`, `String`, `Unit`.
    *   Literals directly provide their types (e.g., `true` is `Bool`, `"abc"` is `String`).
*   **Tuple Types**: `(T1, T2, ...)`
    *   The type of a tuple literal `(e1, e2)` will be inferred as `(type_of(e1), type_of(e2))`. Constraints will be generated accordingly.
*   **Array Types**: `[T]`
    *   The type of an array literal (e.g., `[e1, e2]`, syntax ⚠️ **TBD (AST-LIT)** as per `AST_SPECIFICATION.md`, §10) would require all elements to unify to a common type `T`, making the literal's type `[T]`.
    *   If an empty array literal `[]` is used, its type might need context or default to `[Never]` or `[α]` (a fresh type variable) requiring further constraints.
*   **Function Types**: `fn(T1, T2) -> R`
    *   Inferred for lambda/closure expressions (⚠️ **TBD (AST-LAMBDA)** syntax).
    *   Used to type check function calls and assignments of functions to variables.
*   **Generic Types (Basic Support)**:
    *   The inference machinery must handle type variables that appear within generic type constructors like `Option<T>` or `Result<T, E>` (assuming these are defined in the std-lib with known generic signatures).
    *   Example: If `foo()` returns `Option<α>` and is assigned to `let x = foo();`, then `x` has type `Option<α>`. If `x` is later used in a context expecting `Option<Int>`, then `α` unifies with `Int`.
*   **`_` Wildcard (Gradual Typing)**:
    *   When `_` is used in a type annotation (e.g., `let x: _ = 10;`), it introduces a fresh type variable that must be inferred from context (e.g., from the initializer or later usage).
    *   `let y: List<_> = make_list_of_ints();` implies `_` should be `Int`.

## 4. Interaction with AST Constructs

*   **Literals**: `LiteralNode` directly provides a concrete type (e.g., `1` -> `Int`, `"s"` -> `String`).
*   **Identifiers**: `IdentifierNode` in an expression context is assigned a fresh type variable, which is then constrained by its declaration or usage.
*   **Variable Declarations (`VariableDeclNode`)**:
    *   `let name: T = init;`: Type of `name` is `T`. `type_of(init)` must unify with `T`.
    *   `let name = init;`: Type of `name` is `type_of(init)` (inferred).
    *   `let name: _ = init;`: Type of `name` is a fresh type variable `α`. `α` must unify with `type_of(init)`.
    *   `let name: T;`: Type of `name` is `T`. (Requires later assignment if allowed, or definite assignment analysis. For v0.1, uninitialized `let` might be disallowed or restricted).
*   **Function Declarations (`FunctionDeclNode`)**:
    *   Parameters with type annotations provide known types within the function body.
    *   Return type annotation constrains the type of the function body's result.
    *   If return type is omitted (and allowed), it's inferred from `return` statements.
    *   If parameter types are omitted (⚠️ **TBD** if this is allowed for non-closures), they become fresh type variables.
*   **Function Calls (`FunctionCallNode`)**:
    *   `callee(arg1, arg2)`: If `callee` has type `fn(P1, P2) -> R`, then `type_of(arg1)` must unify with `P1`, `type_of(arg2)` with `P2`, and the call expression has type `R`.
*   **Operators (`UnaryOpNode`, `BinaryOpNode`, `AssignNode` from `ExprKind::Assign`)**:
    *   Operators have predefined type signatures. For `ExprKind::Assign { op, lhs, rhs }`, the type of `lhs` must be compatible with assignment (an l-value), and `type_of(rhs)` must unify with `type_of(lhs)` (or the element type for compound assignments like `+=`). The assignment expression itself evaluates to `Unit` (as per `docs/rfc/RFC-002_CORE_SEMANTICS.md`, §4).
    *   Example: `a + b`. `a` gets type `α`, `b` gets type `β`. If `+` requires operands of the same numeric type `NumT` and returns `NumT`, then constraints `α = NumT`, `β = NumT` are generated. The expression type is `NumT`.
*   **`IfExpression` / `IfStatementNode`**: `cond` must be `Bool`. Both branches must unify to a common type `T`. The `if` expression then has type `T`.
*   **`MatchExprNode`**: Scrutinee expression has type `S`. Each arm `pattern => body` implies `type_of(pattern)` must unify with `S`. All arm bodies must unify to a common type `T`, which is the type of the `match` expression.

## 5. Bidirectional Type Checking/Inference

As suggested in `docs/Steps.md`, Ferra will aim for bidirectional type inference to improve accuracy and error messages. This is the refined, primary strategy for type checking and inference in Ferra.

*   **Checking Mode (Top-Down)**: When the context expects a certain type, this information is pushed down. Example: `let x: Int = expr;`. Here, `expr` is *checked* against type `Int`.
    *   This helps resolve ambiguities (e.g., type of `[]` or numeric literals like `42` can be determined if the context expects `List<String>` or `Float` respectively).
    *   Can provide more targeted error messages.
    *   Example: `let x: Option<Int> = Some(value);`. The context `Option<Int>` is pushed to `Some(value)`. The `Some` constructor (assuming std-lib `Option<T>`) expects an argument of type `T`. Thus, `value` is checked against `Int`, inferring `value` to be `Int` if it was, for instance, an untyped numeric literal like `42` or an expression whose type was previously `α`.
*   **Synthesis Mode (Bottom-Up)**: When no contextual type is available, the type is inferred from the sub-expressions. Example: `let x = 10 + 20;`. Type of `10` is `Int`, `20` is `Int`, so `+` implies result is `Int`, thus `x` is `Int`.
*   **Interaction**: The system will primarily use synthesis but switch to checking mode when type annotations or other contextual information is available. The type-checker will likely have functions like `infer_expr(expr) -> Type` and `check_expr(expr, expected_type)`. 
*   **v0.1 Scope**: For v0.1, the primary goal is to lay the groundwork for row polymorphism, focusing on inference-driven scenarios. Full HM with row polymorphism can be complex.
    *   The system will initially support functions that can operate on records possessing a minimum set of known fields, primarily for read access. For example, a function `fn get_name(record)` could be inferred to work if `record` has a `name: String` field, without needing to know all other fields of `record`.
    *   Structural updates (modifying a field while preserving other unknown fields of a row-polymorphic record) are more complex as they may require language features like record spread/rest operators, and are likely deferred beyond the initial v0.1 implementation unless such operators are included.
    *   Explicit syntax for row variables in type annotations (e.g., `type X = { name: String | OtherFields }`) is not planned for v0.1; inference is key.
    *   The existing AST `TypeKind` for data classes (e.g., `TypeKind::Simple` pointing to a `data` class name) is expected to be sufficient for v0.1. The complexities of row polymorphism will be managed by the type checker's internal representation of types and constraints.
    *   ⚠️ **TBD (TYPE-ROWPOLY-1)**: ~~Define the precise extent and inference mechanism for row polymorphism in v0.1 vs. later versions.~~ The v0.1 extent is focused on inference for functions accepting records with minimal field sets for reading. Advanced updates and explicit row variable syntax are future considerations. Further refinement of inference rules for specific edge cases will occur during implementation.

## 6. Row Polymorphism for Records (`data` classes)

(As per `docs/Steps.md`)
*   **Goal**: To support structural subtyping for records/data classes, allowing functions to operate on records that have *at least* a certain set of fields, without knowing all fields.
*   **Conceptual Model**: A record type like `{name: String, age: Int, ...rest_of_fields}` where `...rest_of_fields` is a row variable.
    *   Example Usage: `fn get_name(record: { name: String, ... }) -> String { return record.name; }`
      This function could accept `data User { name: String, email: String }` or `data Product { name: String, price: Float }` as long as they have a `name: String` field.
*   **Inference Impact**: Hindley-Milner needs to be extended to handle row variables and constraints on them.
    *   Field access `record.fieldName` requires `fieldName` to be present in the record's row type.
    *   Record construction and pattern matching will generate constraints on row variables.
*   **v0.1 Scope**: For v0.1, the primary goal is to lay the groundwork for row polymorphism, focusing on inference-driven scenarios. Full HM with row polymorphism can be complex.
    *   The system will initially support functions that can operate on records possessing a minimum set of known fields, primarily for read access. For example, a function `fn get_name(record)` could be inferred to work if `record` has a `name: String` field, without needing to know all other fields of `record`.
    *   Structural updates (modifying a field while preserving other unknown fields of a row-polymorphic record) are more complex as they may require language features like record spread/rest operators, and are likely deferred beyond the initial v0.1 implementation unless such operators are included.
    *   Explicit syntax for row variables in type annotations (e.g., `type X = { name: String | OtherFields }`) is not planned for v0.1; inference is key.
    *   The existing AST `TypeKind` for data classes (e.g., `TypeKind::Simple` pointing to a `data` class name) is expected to be sufficient for v0.1. The complexities of row polymorphism will be managed by the type checker's internal representation of types and constraints.
    *   ⚠️ **TBD (TYPE-ROWPOLY-1)**: ~~Define the precise extent and inference mechanism for row polymorphism in v0.1 vs. later versions.~~ The v0.1 extent is focused on inference for functions accepting records with minimal field sets for reading. Advanced updates and explicit row variable syntax are future considerations. Further refinement of inference rules for specific edge cases will occur during implementation.

## 7. Error Reporting (Type Errors)

*   When unification fails (type mismatch), provide clear messages indicating:
    *   The conflicting types.
    *   The source locations involved.
    *   The reason for the expected type (e.g., "expected `Int` because of addition with another `Int`").
*   Occurs check failures (infinite types) should also be reported clearly.
*   Unresolved type variables at the end of inference (if not part of a polymorphic generalization at the top level) indicate insufficient information and should be reported as errors.
*   Leverage bidirectional checking to provide context: "Expected type `T` based on [reason], but found type `U` at [location]." The bidirectional approach will be instrumental in making error messages more precise by pinpointing issues more directly within complex expressions (e.g., highlighting a specific incorrect element within a list literal) and providing richer contextual information about type expectations, as illustrated with examples in `FRONTEND_ENHANCEMENTS.MD`.
*   **User-Facing Type Variables**: Type variables (`α`, `β`, etc.) used internally by the inference algorithm **MUST NOT** be shown directly to users in error messages. They should be pretty-printed as anonymous placeholders (e.g., `_`, `_type1`, `inferred_type_of_foo`) or inferred concrete types where possible to improve readability and understanding.
*   **(TYPE-LIT-DEFAULT-1)** Default type resolution for ambiguous literals when no contextual type information is available (e.g., does `let x = 42` default to `Int`? Does `let arr = []` default to `[Never]` or `[α]`?). Bidirectional checking against a contextual type will often resolve this; rules for purely unconstrained literals need to be firm.
*   Interaction with lifetime inference (if/when explicit lifetimes are added, see §8).

## 8. Limitations & Extensions (v0.1)

*   **Higher-Rank Types**: Full higher-rank polymorphism (e.g., `fn(f: (forall a. a -> a))`) is typically beyond standard HM and is deferred (see §2 for Rank-1 scope).
*   **Trait System / Type Classes**: A full trait system for ad-hoc polymorphism is a major extension. For v0.1, operators might have a fixed set of overloaded signatures for built-in types.
*   **Effects System**: Tracking effects (e.g., I/O, exceptions) in the type system is out of scope for v0.1.
*   **Lifetime Inference**: While ownership and borrowing are core, the detailed interaction of type inference with a full lifetime inference system (beyond basic scope-based reasoning) is closely tied to the ownership model and will be further detailed in conjunction with the Ownership Model RFC (see `docs/rfc/RFC-003_OWNERSHIP_PRINCIPLES.md`).
*   **Monomorphisation Details**: While the goal is Rust-class monomorphisation (see §1), the precise mechanics and control over instantiation (e.g., explicit vs. automatic) for all generic constructs are details for later phases, building upon a successful type inference pass.

## 9. Open Questions & Future Considerations (Type Inference Specific)

*   **(TYPE-ROWPOLY-1)** Precise specification and implementation complexity of row polymorphism for v0.1.
*   **(TYPE-ADHOC-POLY-1)** Strategy for operator overloading and ad-hoc polymorphism if a full trait system is deferred beyond v0.1.
*   **(TYPE-ERROR-MSG-1)** Collection of good/bad type error message examples to guide implementation and ensure "positive-first" principles are upheld for type errors.
*   **(TYPE-LIT-DEFAULT-1)** Default type resolution for ambiguous literals when no contextual type information is available (e.g., does `let x = 42` default to `Int`? Does `let arr = []` default to `[Never]` or `[α]`?). Bidirectional checking against a contextual type will often resolve this; rules for purely unconstrained literals need to be firm.
*   Interaction with lifetime inference (if/when explicit lifetimes are added, see §8).
*   Type inference for macros.
*   Performance of the unification algorithm for large constraint sets.
*   Details of type generalization and instantiation for polymorphic `let` bindings (e.g., value restriction, handling of mutable vs. immutable bindings for generalization).
*   Handling of recursive type definitions and their impact on inference (e.g., for `data` types).
*   Precise rules for type inference of literals when context is ambiguous (e.g., should `let x = []` default to `[Never]` or remain `[α]` until constrained?). 