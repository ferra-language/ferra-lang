# Ferra Front-End Enhancements Design (Post v0.1 Refinements)

This document details planned enhancements to the Ferra compiler front-end, building upon the initial designs for the lexer (`DESIGN_LEXER.md`), parser (`DESIGN_PARSER.md`), AST (`AST_SPECIFICATION.md`), type inference (`DESIGN_TYPE_INFERENCE.md`), and diagnostics (`DESIGN_DIAGNOSTICS.md`).

These enhancements correspond to **Module 2.5** in the `comprehensive_plan.md`.

## 1. Row Polymorphism for Records (Step 2.5.1)

**Goal**: Extend the type system to support row polymorphism for record types (Ferra `data` classes), allowing functions to operate on records that have *at least* a specified set of fields, without knowing all fields. This increases flexibility and code reuse.

**References**:
*   `DESIGN_TYPE_INFERENCE.md` (§6)
*   `AST_SPECIFICATION.md` (PatternKind::DataDestruct, TypeKind::Simple for data classes)
*   `Steps.md` (§1, Type Inference)

**Design Details**:

*   **Conceptual Model**:
    *   A record type like `{name: String, age: Int | r}` where `r` is a row variable representing the rest of the fields.
    *   A function `fn process(rec: {name: String | r}) -> String` can accept any record that has at least a `name: String` field.
*   **Type Inference Extension**:
    *   The Hindley-Milner based type inference system (`DESIGN_TYPE_INFERENCE.md`, §2, §6) will be extended to handle row variables.
    *   Constraints will be generated for:
        *   Field access (`record.field`): `field` must be present in the row.
        *   Record construction: The constructed record's type includes all specified fields and potentially a fresh row variable for extensibility or a closed row if fully specified.
        *   Pattern matching on records (`PatternKind::DataDestruct`): Constraints will unify the scrutinee's row with the pattern's expected fields and row variable.
*   **Syntax**:
    *   No new syntax is strictly required for function parameters if type inference can deduce row polymorphism.
    *   Type annotations might eventually allow explicit row variables (e.g., `data Point {x: Int, y: Int | Rest}`), but this is a more advanced feature. Initially, inference will be key.
*   **AST Impact**:
    *   `TypeKind` might need a variant for explicit row types if they become a surface syntax feature. Otherwise, the existing `TypeKind::Simple` (for data class names) will be associated with inferred row-polymorphic types by the type checker.
    *   The type representation within the type checker will need to accommodate row variables.
*   **Challenges**:
    *   Unification with row variables can be complex, especially with multiple row variables or overlapping field sets.
    *   Clear error messages for row-related type mismatches.
*   **Implementation Strategy**:
    1.  Research and adapt existing algorithms for HM with row polymorphism (e.g., from languages like OCaml, SML, or Elm, or relevant academic papers).
    2.  Extend the unification engine in the type checker.
    3.  Focus initially on inference for function arguments and record literals.
    4.  Gradually enhance pattern matching support.

### 1.1. Usage Examples

Let's illustrate how row polymorphism would work in practice.

**Example 1: Processing records with common fields**

Consider functions that operate on any record containing `name: String` and `age: Int`.

```ferra
// Ferra `data` definitions
data Person {
    name: String,
    age: Int,
    city: String,
}

data Employee {
    name: String,
    age: Int,
    employee_id: String,
    department: String,
}

data Pet {
    name: String,
    species: String,
    // No age field
}

// A function accepting any record with at least `name: String`
// The type `{ name: String | r }` is conceptual for the type system;
// Ferra might infer this without explicit syntax in the function signature for v0.1.
// fn print_name(record: { name: String | r }) {
// For v0.1, we rely on inference. So the signature might look like:
fn print_name(record_with_name) { // Type system infers `record_with_name` must have a `.name` field of type String
    println("Name: " + record_with_name.name);
}

// A function accepting any record with at least `name: String` and `age: Int`
fn print_name_and_age(record_with_details) { // Inferred to require `.name: String` and `.age: Int`
    println("Name: " + record_with_details.name + ", Age: " + String::from_int(record_with_details.age)); // `String::from_int` from STDLIB_CORE_V0.1.md §3.1
}

fn main() {
    let person = Person { name: "Alice", age: 30, city: "New York" };
    let employee = Employee { name: "Bob", age: 42, employee_id: "E123", department: "Engineering" };
    let pet = Pet { name: "Buddy", species: "Dog" };

    print_name(person);     // OK: Person has `name: String`
    print_name(employee);   // OK: Employee has `name: String`
    print_name(pet);        // OK: Pet has `name: String`

    print_name_and_age(person);   // OK: Person has `name` and `age`
    print_name_and_age(employee); // OK: Employee has `name` and `age`
    // print_name_and_age(pet);    // COMPILE ERROR: Pet does not have `age: Int`
}
```

This illustrates how `print_name` can operate on various record types as long as they satisfy the structural requirement of having a `name` field. The type system, through row polymorphism, would ensure this.

**Example 2: Updating records structurally**

Imagine a function that takes a record, updates a specific field if present, and returns a new record of the same "shape" (including any other fields).

```ferra
// Conceptual: `R` is a type variable representing a record type with at least `count: Int`.
// The `| r` denotes the row variable for other fields.
// fn increment_count<R where R is { count: Int | r }>(record: R) -> R {
// Simpler v0.1 relying on inference:
fn increment_count(record_with_count) { // Inferred: input must have `.count: Int`, output is same structural type
    // This is tricky. To preserve other fields not known to `increment_count`,
    // the language would need a way to construct a new record with one field modified
    // while copying the rest. This might require spread/rest operators for records:
    // return { ...record_with_count, count: record_with_count.count + 1 }; // Syntax TBD
    // For v0.1, if such operators aren't available, functions might be restricted
    // to only *reading* from row-polymorphic records, or the specific mechanism
    // for "updating" while preserving unknown fields needs to be defined.
    //
    // Alternative for v0.1 (less flexible, requires known type):
    // If the function knows the concrete type, or if it creates a NEW specific type:
    // return UpdatedCounter { original_id: record_with_count.id, new_count: record_with_count.count + 1 };

    // For simplicity in v0.1, let's assume field access for reading is the primary use case.
    // True structural update with unknown fields is an advanced aspect.
    // For now, this function will just read.
    println("Count was: " + String::from_int(record_with_count.count));
    // No return for this simplified example.
}

data Widget { id: String, count: Int, color: String }
data Gadget { name: String, count: Int }

fn main() {
    let w = Widget { id: "w1", count: 10, color: "Red" };
    let g = Gadget { name: "g1", count: 5 };

    increment_count(w);
    increment_count(g);
}

```
The `increment_count` example highlights that updating fields while preserving unknown fields in a truly row-polymorphic way requires careful consideration of record construction syntax (e.g., spread/rest operators), which might be beyond the initial v0.1 scope if not explicitly planned. The primary v0.1 benefit might be in functions that *read* from records with a minimum set of fields.

### 1.2. Error Reporting Examples

**Scenario 1: Missing Field**

*   **Code Example**:
    ```ferra
    data Product { name: String, price: Float }
    data Animal { name: String, species: String }

    fn print_price(item_with_price) { // Expects `.price: Float`
        println("Price: " + String::from_float(item_with_price.price)); // Assuming `String::from_float` analogous to `String::from_int` in STDLIB_CORE_V0.1.md §3.1
    }

    fn main() {
        let apple = Product { name: "Apple", price: 1.99 };
        let dog = Animal { name: "Dog", species: "Canine" };
        print_price(apple); // OK
        print_price(dog);   // ERROR
    }
    ```
*   **Conceptual Diagnostic Output**:
    ```text
    error: missing field `price` for expression [E3XX] // E3XX is a placeholder type error code
      --> main.ferra:11:19
        |
    4   |     fn print_price(item_with_price) { // Expects `item_with_price` to have field `.price`
        |        ----------- function `print_price` called here
    ...
    11  |         print_price(dog);   // ERROR
        |                     ^^^ this expression is of type `Animal`
        |
    hint: The function `print_price` expects an argument with a field named `price` of type `Float`.
          Type `Animal` (from `dog`) is missing the field `price`.
          Ensure the value passed to `print_price` has a `price: Float` field, or define it on `Animal`.
    ```

**Scenario 2: Field Type Mismatch**

*   **Code Example**:
    ```ferra
    data Item { id: String, count: Int }
    data Stock { id: String, count: String } // count is String here

    fn process_item_count(record_with_int_count) { // Expects `.count: Int`
        if record_with_int_count.count > 0 {
            println("Count is positive.");
        }
    }

    fn main() {
        let item = Item { id: "A1", count: 10 };
        let stock = Stock { id: "B2", count: "twenty" };
        process_item_count(item);  // OK
        process_item_count(stock); // ERROR
    }
    ```
*   **Conceptual Diagnostic Output**:
    ```text
    error: mismatched types for field `count` [E3XY]
      --> main.ferra:13:26
        |
    4   |     fn process_item_count(record_with_int_count) { // Expects `record_with_int_count.count` to be `Int`
        |        ------------------ function `process_item_count` called here
    5   |         if record_with_int_count.count > 0 {
        |            ---------------------------- field `count` accessed here, expected type `Int` for comparison with `0`
    ...
    13  |         process_item_count(stock); // ERROR
        |                            ^^^^^ this expression is of type `Stock`
        |
    hint: The function `process_item_count` expects the field `count` to be of type `Int`.
          However, type `Stock` (from `stock`) has a field `count` of type `String`.
          Consider converting `stock.count` to `Int` before calling, or ensure the `count` field has the correct type.
    ```

These examples aim to provide positive guidance ("expects an argument with a field...") before stating the problem ("missing the field" or "mismatched types").

### 1.3 Conceptual Syntax for Explicit Row Types (Future Consideration)

While Ferra v0.1 aims for inference-driven row polymorphism, future versions might introduce explicit syntax for defining or annotating with row types. A conceptual EBNF for such syntax could be:

```ebnf
RowType       ::= "{" FieldList ( ","? "|" RowVar )? (",")? "}" (* Allowing trailing comma and optional comma before pipe *)
                | "{" "|" RowVar "}" (* A type that is only a row variable *)
FieldList     ::= FieldDecl ("," FieldDecl)*
FieldDecl     ::= IDENTIFIER ":" Type
RowVar        ::= IDENTIFIER (* Represents the variable for the rest of the fields *)
```

*   **Example**: `let process_partial = fn(record: { name: String, age: Int | r }) -> String { ... }`
*   This syntax would allow developers to be more explicit in type signatures if needed. For v0.1, the type checker would internally manage similar concepts without this surface syntax.

## 2. Fully Bidirectional Type Inference (Step 2.5.2)

**Goal**: Refine the type inference system to be fully bidirectional, enhancing its power, the clarity of error messages, and its ability to handle more complex type-level constructs smoothly.

**References**:
*   `DESIGN_TYPE_INFERENCE.md` (§5)
*   `Steps.md` (§1, Type Inference: "bidirectional for readable errors")

**Design Details**:

*   **Recap of Bidirectional Inference**:
    *   **Checking Mode (Top-Down)**: An expression `e` is checked against an expected type `T`. `check(e, T)`.
    *   **Synthesis Mode (Bottom-Up)**: The type of an expression `e` is inferred without prior context. `infer(e) -> T`.
*   **Refinement Goals**:
    *   **Systematic Application**: Ensure that the choice between checking and synthesis is applied systematically throughout all expression and statement forms in the language.
    *   **Improved Error Locus**: Bidirectional checking can often pinpoint errors more precisely by propagating type expectations. For example, in `let x: List<Int> = [1, "two"];`, the `List<Int>` expectation is pushed into the array literal, allowing an error to be reported directly on `"two"` not matching `Int`.
    *   **Enhanced Inference for Ambiguous Constructs**:
        *   Empty literals (e.g., `[]`, `%{}` for maps if added): Contextual type from checking mode can resolve their type.
        *   Numeric literals: `let x: Float = 42;` (checking `42` against `Float`).
        *   Lambda expressions without full annotations: `let f: fn(Int)->Int = (x) => x + 1;` (checking `(x) => x + 1` against `fn(Int)->Int`, inferring `x` as `Int`).
*   **AST Traversal**:
    *   The type checker's main traversal functions will be explicitly designed around `check_expr(expr_id, expected_type)` and `infer_expr(expr_id) -> inferred_type`.
    *   Rules for each AST node kind (`ExprKind`, `StmtKind`) will specify when to switch modes.
        *   Example (Function Call `f(a, b)`):
            1.  Infer type of `f` -> `fn(T1, T2) -> Tr`.
            2.  Check `a` against `T1`.
            3.  Check `b` against `T2`.
            4.  Resulting type is `Tr`.
        *   Example (Variable Declaration `let x: T = e;`):
            1.  Check `e` against `T`.
*   **Interaction with Other Features**:
    *   **Row Polymorphism**: Bidirectional inference can help guide the inference of row variables by providing expected structural types.
    *   **Generics**: Crucial for instantiating generic functions or types based on context.
*   **Implementation Strategy**:
    1.  Review and refactor existing type inference logic (`DESIGN_TYPE_INFERENCE.md`) to strictly follow bidirectional principles.
    2.  Define explicit `check` and `infer` rules for every relevant AST node.
    3.  Augment test suites with cases that specifically benefit from bidirectional checking for both successful inference and error reporting.

#### 2.1. AST Traversal Examples (Expanded)

The core idea is to have `check_expr(expr_id, expected_type)` and `infer_expr(expr_id) -> InferredType` functions. Here's how they might apply to more constructs:

*   **Example (If Expression `if cond { then_branch } else { else_branch }`)**:

    *   **Checking Mode**: `check_expr(if_expr_id, expected_type)`
        1.  Call `check_expr(cond_id, Bool_Type)`. The condition must be boolean.
        2.  Call `check_expr(then_branch_id, expected_type)`.
        3.  Call `check_expr(else_branch_id, expected_type)`.
        4.  The `then_branch` and `else_branch` must successfully check against `expected_type`.

    *   **Synthesis Mode**: `infer_expr(if_expr_id) -> InferredType`
        1.  Call `check_expr(cond_id, Bool_Type)`.
        2.  Call `infer_expr(then_branch_id) -> type_A`.
        3.  Call `infer_expr(else_branch_id) -> type_B`.
        4.  Unify `type_A` and `type_B` to find a common supertype `type_C`. This `type_C` is the inferred type of the `if` expression. If they don't unify, it's a type error.

*   **Example (Match Expression `match scrutinee { pat1 => arm1, pat2 => arm2, ... }`)**:

    *   **Checking Mode**: `check_expr(match_expr_id, expected_type)`
        1.  Call `infer_expr(scrutinee_id) -> scrutinee_type`.
        2.  For each arm `pat_i => arm_i`:
            a.  Perform pattern matching type check: `check_pattern(pat_i_id, scrutinee_type)` (this infers types for variables bound in `pat_i`).
            b.  Call `check_expr(arm_i_id, expected_type)` within the scope extended by bindings from `pat_i`.
        3.  All arms must successfully check against `expected_type`.

    *   **Synthesis Mode**: `infer_expr(match_expr_id) -> InferredType`
        1.  Call `infer_expr(scrutinee_id) -> scrutinee_type`.
        2.  Initialize `common_arm_type = Unresolved_Type_Variable`.
        3.  For each arm `pat_i => arm_i`:
            a.  `check_pattern(pat_i_id, scrutinee_type)`.
            b.  `infer_expr(arm_i_id) -> current_arm_type`.
            c.  Unify `common_arm_type` with `current_arm_type`. If unification fails, it's a type error (arms have incompatible types). Update `common_arm_type` with the result of unification.
        4.  The final `common_arm_type` (if resolved) is the inferred type of the `match` expression.

*   **Example (Lambda/Closure Expression `(param1, param2) => body_expr`)** (Syntax TBD, using a conceptual form):

    *   **Checking Mode**: `check_expr(lambda_id, expected_fn_type: fn(P1, P2) -> R)`
        1.  Infer types for `param1`, `param2` from `P1`, `P2` respectively.
        2.  In the scope extended with `param1: P1`, `param2: P2`, call `check_expr(body_expr_id, R)`.

    *   **Synthesis Mode**: `infer_expr(lambda_id) -> InferredFnType: fn(T1, T2) -> Tr`
        1.  Assign fresh type variables `alpha1`, `alpha2` to `param1`, `param2`.
        2.  In the scope extended with `param1: alpha1`, `param2: alpha2`, call `infer_expr(body_expr_id) -> body_type`.
        3.  The inferred type of the lambda is `fn(alpha1, alpha2) -> body_type`. These type variables (`alpha1`, `alpha2`, and those within `body_type`) may be further constrained or resolved by the context in which the lambda is used. For instance, if the lambda is immediately called or assigned to a variable with a more specific function type annotation.

#### 2.2. Illustrating Error Message Improvement

Bidirectional inference can provide much more targeted error messages.

**Scenario**: A function expects a `List<Int>`, but is given a list containing a string.

*   **Code Example**:
    ```ferra
    fn process_numbers(numbers: List<Int>) {
        // ... processes numbers ...
        if !numbers.is_empty() {
            println("First number: " + String::from_int(numbers.get(0).unwrap_or_default()));
        }
    }

    fn main() {
        let mixed_list = [1, 2, "error_here", 4]; // Assuming array literal `[...]` infers to `List<T>`
        process_numbers(mixed_list); // Error occurs here
    }
    ```

*   **Conceptual Error Without Strong Bidirectional Checking (Simplified HM style)**:
    The error might only be caught when `process_numbers` is called, and the message might be generic.
    ```text
    error: mismatched types [E3AB] // E3AB is a placeholder type error code
      --> main.ferra:10:23
        |
    10  |         process_numbers(mixed_list);
        |                         ^^^^^^^^^^ expected `List<Int>`, found `List<[Int | String]>` (or similar unified type)
        |
    note: `mixed_list` was inferred to have type `List<[Int | String]>` due to its elements.
    ```
    This error points to the call site, and the inferred type `List<[Int | String]>` (or a more complex union/inference variable) might not clearly pinpoint *which* element caused the issue.

*   **Conceptual Error WITH Bidirectional Checking**: 
    When `process_numbers(mixed_list)` is type-checked, the `List<Int>` expectation is pushed *down* into the `mixed_list` during its type inference/checking.
    ```text
    error: mismatched types: expected `Int` but found `String` [E3AC] // E3AC is a placeholder type error code
      --> main.ferra:9:31
        |
    9   |         let mixed_list = [1, 2, "error_here", 4];
        |                                 ^^^^^^^^^^^^ this element is of type `String`
        |
    note: Argument `numbers` of function `process_numbers` expects type `List<Int>`.
      --> main.ferra:1:30
        |
    1   |     fn process_numbers(numbers: List<Int>) {
        |                                 ^^^^^^^^^ expected `List<Int>` here
    hint: All elements of the list passed to `process_numbers` must be of type `Int`.
          The element `"error_here"` at index 2 is a `String`.
    ```
    This error is much more precise:
    1.  It points directly to the problematic element (`"error_here"`) within the list literal.
    2.  It clearly states the expected element type (`Int`) due to the function's parameter type.
    3.  It shows where the expectation (`List<Int>`) originated.

#### 2.3. Impact on `DESIGN_TYPE_INFERENCE.md`

The refinement to a "fully bidirectional" system implies:
*   **Strengthening Section 5 ("Bidirectional Type Checking/Inference")**: This section in `DESIGN_TYPE_INFERENCE.md` becomes the central specification for the type inference algorithm. The principles laid out there (checking vs. synthesis, interaction) need to be rigorously applied to all language constructs.
*   **Clarifying Rules for Each AST Node**: For each `ExprKind` and relevant `StmtKind` in `AST_SPECIFICATION.md`, the specific bidirectional rules (when to switch to `check` mode vs. `infer` mode, and what type information is passed down or synthesized up) need to be documented, likely as an extension or appendix to `DESIGN_TYPE_INFERENCE.md` or within the implementation itself with clear comments.
*   **Resolving Ambiguities**: Explicitly document how bidirectionality resolves ambiguities mentioned in `DESIGN_TYPE_INFERENCE.md` (e.g., type of `[]`, numeric literals).
*   **Error Reporting Focus**: The goal of "readable errors" from `Steps.md` (§1) via bidirectionality needs to be emphasized, with `DESIGN_TYPE_INFERENCE.md` (§7 Error Reporting) updated to reflect how bidirectional information improves error messages.

This refinement doesn't necessarily change the fundamental HM algorithm but rather systematizes its application and leverages contextual type information more effectively for better inference and diagnostics.

## 3. Bloom-Filter De-duplication for Diagnostics (Step 2.5.3)

**Goal**: Implement a mechanism, such as a Bloom filter or a similar probabilistic data structure, to de-duplicate diagnostic messages, especially errors, to avoid overwhelming the user with redundant or closely related messages originating from a single root cause or physical location. Target ≤ 3 messages per line as per `Steps.md`.

**References**:
*   `DESIGN_DIAGNOSTICS.md` (§8)
*   `Steps.md` (§1, Diagnostics: "Bloom-filter de-duplication → ≤ 3 messages/line")

**Design Details**:

*   **Problem**: A single syntax or type error can cascade, causing many subsequent errors. Reporting all of them is unhelpful.
*   **Mechanism**:
    1.  **Key Generation**: For each diagnostic, generate a key. This key could be a tuple or hash of:
        *   Diagnostic code (e.g., `E0077`).
        *   Primary span (file, line, and possibly a summarized column or the start of the span).
        *   Possibly a snippet of the source text or a hash of the primary message.
    2.  **Bloom Filter**:
        *   Before reporting a diagnostic, add its key to a Bloom filter.
        *   If the filter indicates the key (or a similar key, if using locality-sensitive hashing) might already be present, consider suppressing the new diagnostic or marking it as related to a previously reported one.
    3.  **Per-Line Counter**: Maintain a counter for diagnostics reported per line. If this exceeds a threshold (e.g., 3), subsequent diagnostics on the same line might be suppressed or summarized.
*   **Alternative/Complementary Heuristics**:
    *   **Error Distance**: If a new error is within a very small span of a previously reported error of a similar kind, it might be a duplicate.
    *   **Parser/Checker State**: If the parser or type checker is in an error recovery mode, it might be more aggressive in suppressing diagnostics until it's confident it has resynchronized.
*   **Implementation Strategy**:
    1.  Choose the primary de-duplication data structure (Bloom filter is suggested, but alternatives can be evaluated for simplicity vs. accuracy).
    2.  Integrate into the diagnostic reporting pipeline (`DESIGN_DIAGNOSTICS.MD`).
    3.  When a diagnostic is about to be emitted, query the de-duplication mechanism.
    4.  Define clear criteria for what constitutes a "duplicate" (e.g., exact same error code on the same line, or same type of error within N characters/tokens).
    5.  Provide a mechanism (perhaps internal compiler flags) to disable de-duplication for debugging the compiler itself.
*   **Configuration**: The limit (e.g., 3 messages per line) should ideally be configurable. Specific CLI flags for managing de-duplication behavior (e.g., `--deduplication=off|on`, `--deduplication-fp-rate=0.01`) and associated environment variables (e.g., `FERRA_DEDUPLICATION_LEVEL`) would be detailed in the compiler's main CLI specification or in `DESIGN_DIAGNOSTICS.md` where other diagnostic-related flags are discussed.

#### 3.1. Key Generation for De-duplication

Effective de-duplication relies on a good key that can identify "similar" or "identical" diagnostics. The key should be a composite of several pieces of information derived from the structured diagnostic object (see `DESIGN_DIAGNOSTICS.md`, §3, especially §3.2 for the JSON Line Protocol which details fields like `code`, `span`, etc.):

1.  **Diagnostic Code**: The unique error code (e.g., `E0077`, `E3AC`) is a primary component. This groups errors of the same kind.
2.  **Primary Source Location (Granular)**:
    *   `file_id`: Essential for distinguishing errors in different files.
    *   `line_number`: The most critical part of the location.
    *   `column_number_bucket`: Instead of the exact column, using a "bucket" (e.g., rounding down to the nearest 4 or 8 columns, or simply using the start column of the primary span) can help group errors that are very close horizontally on the same line and likely related. Exact column matching might be too specific and miss slightly shifted but essentially identical cascaded errors.
3.  **Primary Message Summary (Optional but Recommended)**:
    *   A hash of a normalized version of the primary diagnostic message string. Normalization might involve lowercasing, removing specific variable names (replacing with a placeholder like `_VAR_`), or taking the first N characters/words. This helps differentiate errors with the same code and location but subtly different messages (though this should be rare if codes are granular).
    *   Alternatively, if error messages are highly templated based on the error code, the code itself might be sufficient without hashing the message.

**Key Composition Example**:
A string key would be formed using a pattern like `<ErrorCode>@<FileID>:<Line>:<ColBucket>(:<MsgHash>)`.
For example, a key value could be the string `"E3AC@main.ferra:9:28:abcdef12"` (assuming column 28-31 bucket to 28, and `abcdef12` is a hash of the message summary).

The goal is to make keys identical for diagnostics that are effectively duplicates from the user's perspective, while being different enough for genuinely distinct issues.

#### 3.2. Bloom Filter Properties and Alternatives

*   **Bloom Filter Properties**:
    *   **Space Efficiency**: Bloom filters are very space-efficient for representing a set.
    *   **Probabilistic**: They can produce false positives (i.e., report that an item *might* be in the set when it's not), but **never false negatives** (if it says an item is not in the set, it's definitely not).
    *   **False Positive Impact**: For diagnostic de-duplication, a false positive means a unique diagnostic might occasionally be suppressed. The rate can be tuned by the filter's size and number of hash functions. A low false positive rate is desirable (e.g., < 1%).
    *   **No Deletion**: Standard Bloom filters do not support item deletion. This is generally fine for a single compilation pass, as the filter is rebuilt each time.

*   **Alternatives and Complements**:
    *   **Hash Set**: Using a standard hash set to store the generated keys.
        *   *Pros*: No false positives. Conceptually simpler.
        *   *Cons*: Higher memory usage compared to a Bloom filter for the same number of items and collision probability.
    *   **Simple Heuristics (as mentioned)**:
        *   **Error Distance**: Suppress if error E2 is same type as E1 and within N tokens/chars on same/next line.
        *   **Parser/Checker State**: If in "panic/recovery" mode, be more aggressive in suppressing further errors from the same region until synchronized.
    *   **Combined Approach**: A Bloom filter can be the first quick check. If it indicates a potential duplicate, a more precise (but potentially more expensive) check or heuristic could be applied.

Given the goal of performance and the tolerance for a very small chance of suppressing a unique error (if the false positive rate is low), a Bloom filter remains a strong candidate. If memory is less of a concern than the small false positive rate, a direct hash set of keys is a viable alternative.

#### 3.3. Interaction of Per-Line Counter with De-duplication Key

The per-line counter (e.g., max 3 messages per line) acts as a secondary throttling mechanism.

1.  **Initial Check**: When a diagnostic `D` is generated for line `L` with key `K`:
    a.  The system first checks the per-line counter for `L`. If `count[L] >= MAX_PER_LINE_DIAGS` (e.g., 3), diagnostic `D` is suppressed (or queued for a summary report later).
    b.  If `count[L] < MAX_PER_LINE_DIAGS`:
        i.  Query the Bloom filter (or hash set) with key `K`.
        ii. If `K` is found (or likely found in Bloom filter), suppress `D` as a duplicate.
        iii. If `K` is not found:
            *   Report diagnostic `D`.
            *   Add `K` to the Bloom filter/hash set.
            *   Increment `count[L]`.

This ensures that even if multiple *distinct* (different keys) errors occur on the same line, the user isn't overwhelmed by more than `MAX_PER_LINE_DIAGS` of them. The Bloom filter handles de-duplication of the *same logical error* that might be reported multiple times due to code structure or error recovery attempts.

The `MAX_ERRORS_PER_FILE` (e.g., 50, as mentioned in `DESIGN_DIAGNOSTICS.md`) acts as a global cutoff, overriding both the per-line counter and Bloom filter once reached.

## 4. `ai::explain(err)` Integration (Step 2.5.4)

**Goal**: Design the compiler's diagnostic output and internal structures to effectively support and integrate with an `ai::explain(err)` API, which would provide AI-powered, natural-language explanations and suggestions for compiler errors.

**References**:
*   `DESIGN_DIAGNOSTICS.md` (§2, §3, §10)
*   `Steps.md` (§1, Diagnostics: "Pipelined into `ai::explain(err)`")
*   `AI_API_AST.md` (for general AI API integration patterns)

**Design Details**:

*   **Structured Diagnostics**: The diagnostic system (`DESIGN_DIAGNOSTICS.md`, §3) already specifies a rich, structured format for diagnostics (severity, code, message, spans, labels, notes, hints). This is the primary input for `ai::explain(err)`.
*   **Compiler Output for AI**:
    *   When `ai::explain(err)` is invoked (potentially triggered by a specific error code or a user action in an IDE), the compiler needs to provide the relevant diagnostic object(s) in a machine-readable format (e.g., JSON).
    *   This might involve the compiler having a mode to emit detailed diagnostic information for a specific error instance.
*   **Information to be Passed to `ai::explain(err)`**:
    *   The full structured diagnostic object.
    *   The source code snippet related to the primary span and any labels.
    *   Potentially, limited contextual information from the compiler's state if safe and useful (e.g., expected type vs. found type, relevant symbol table entries if the error is name-resolution related). This needs careful design to avoid exposing excessive internal compiler details.
    *   The error code (e.g., `E0077`) is crucial for the AI to look up specific error patterns.
*   **AI Responsibilities (Conceptual)**:
    *   Parse the structured diagnostic.
    *   Use the error code and message as a primary key.
    *   Analyze the source snippet and contextual information.
    *   Generate a more verbose, pedagogical explanation.
    *   Suggest alternative fixes, common pitfalls, or links to documentation.
    *   Translate compiler jargon into simpler terms.
*   **Compiler-Side API (Conceptual)**:
    *   The compiler might not directly call an `ai::explain` *function*. Instead, an external tool or IDE component would:
        1.  Receive structured diagnostics from the compiler (e.g., via LSP or JSON output).
        2.  Identify an error instance to be explained.
        3.  Invoke the `ai::explain(err_payload)` API (which could be a separate service or library), passing the structured error data.
*   **Ensuring Effectiveness**:
    *   **Stable Error Codes**: `diagnostic_codes.md` becomes very important.
    *   **Rich Spans and Labels**: Accurate and multiple spans help the AI understand the context.
    *   **Semantic Hints**: The `notes` and `hints` fields in the diagnostic object can provide valuable, curated information that the AI can build upon.
*   **Implementation Strategy**:
    1.  Ensure the JSON output for diagnostics (as per `DESIGN_DIAGNOSTICS.md` §3.2 and §9) is comprehensive and includes all necessary fields from the internal diagnostic schema.
    2.  Develop a clear specification for the data payload expected by the `ai::explain(err)` API endpoint/function.
    3.  Create a corpus of example errors and their ideal structured diagnostic representations to test the pipeline.
    4.  (Future) Potentially add an internal compiler command to dump the full diagnostic context for a given error ID or location, facilitating testing of the AI explanation pipeline.

*   **Note on Attribute Syntax**: The `ai.assume(...)` tags, which might indirectly influence diagnostics by changing compiler assumptions (e.g., for borrow checking as per `OWNERSHIP_BORROW_CHECKER.md`), would leverage Ferra's general attribute syntax already defined in `SYNTAX_GRAMMAR_V0.1.md` (Section 1.3.5). The `ai::explain(err)`