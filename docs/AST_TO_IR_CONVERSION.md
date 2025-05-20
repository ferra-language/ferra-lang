# AST to IR Conversion

## 1. Introduction

This document specifies the process of converting from Ferra's Abstract Syntax Tree (AST) to its Static Single Assignment (SSA) Intermediate Representation (IR). It details the mapping between high-level language constructs and the lower-level IR operations defined in the `IR_SPECIFICATION.md` document.

### 1.1 Purpose

The AST to IR conversion is a critical phase in the Ferra compiler pipeline that:
- Translates the high-level AST into a form suitable for optimization
- Implements SSA construction, ensuring each variable is assigned exactly once
- Performs necessary type conversions and validation
- Establishes a foundation for subsequent optimization passes

### 1.2 Process Overview

The conversion process follows these general steps:
1. Creation of an IR module corresponding to the compilation unit
2. Function-by-function translation of AST nodes to IR instructions
3. Basic block formation based on control flow
4. SSA construction using algorithms like minimal SSA or pruned SSA
5. Type checking and insertion of necessary conversions
6. Metadata attachment for debugging and AI-assisted features

### 1.3 Pipeline Diagram

```
AST  ──▶  HirLower (scope+name resolution) ──▶  SsaBuilder ──▶  IR Module
```

* **HirLower** collapses syntactic sugar (e.g., `if expr` shortcut) into a *HIR* still using AST IDs
* **SsaBuilder** walks HIR, constructs IR functions, basic blocks, and SSA values, inserting `phi` where required

## 2. Environment Structures

The conversion process utilizes several key data structures:

```rust
struct FuncCtx<'a> {
    ir_fn: FunctionBuilder<'a>,          // helper emitting into Function
    scope: ScopeStack,                  // maps IdentId → current ValueIdx
    break_stack: Vec<BlockIdx>,         // targets for `break`
    continue_stack: Vec<BlockIdx>,      // targets for `continue`
}
```

## 3. AST to IR Mapping

### 3.1 Top-Level Constructs

| AST Node Type | IR Conversion Strategy |
|---------------|------------------------|
| Module | Create an IR Module with the same name |
| Function Declaration | Create an IR Function with parameters and return type |
| Type Declaration | Create type definitions in the IR module |
| Import Statement | Create import entries in the IR module |
| Global Variable | Create global variable in the IR module |

### 3.2 Expressions

*Note: This table lists every `ExprKind` in `AST_SPECIFICATION.md`; anything absent is a bug.*

| AST Expression | IR Conversion Strategy |
|----------------|------------------------|
| Literal (Int) | Create an integer constant (`iconst`) |
| Literal (Float) | Create a floating-point constant (`fconst`) |
| Literal (Bool) | Create a boolean constant (`boolconst`) |
| Literal (String) | Create a string constant (`strconst`) |
| Literal (Char) | Create a 32-bit integer constant (`iconst`) representing Unicode codepoint |
| Variable Reference | Load the variable if needed (from stack or global) |
| Binary Operation | Translate to corresponding IR arithmetic/logic instruction |
| Unary Operation | Translate to corresponding IR unary instruction (e.g., `neg` for `-`, `lnot` for `!`) |
| Function Call | Generate a call instruction with arguments |
| FieldAccess (`base.field`) | Get field address using `getfieldptr %base, field_name/index`, then `load` the value. |
| Array/Collection Operation | Generate appropriate memory and access instructions |
| Tuple Creation | Generate instructions to create and populate a tuple |
| If Expression | Create conditional branches and phi nodes as needed |
| Match Expression | Create a series of comparisons and branches with phi nodes |
| Range Expression (`..`, `..=`) | Emit call to builtin `range_new` that returns an iterator object (relates to IR-LOOP-VAR-1) |
| Coalesce (`??`) | Lower to conditional branch: if LHS is non-null jump to merge block, else evaluate RHS; merge block gets `phi(LHS, RHS)` |
| Await | Currently produce the SSA value of its operand unchanged; async state-machine lowering deferred to Phase 2 |
| Error Propagate (`?`) | Lower to: evaluate operand → check for error → branch to either propagate (return err) or continue (unwrap ok) (relates to IR-EXCEPT-1) |

### 3.3 Statements

| AST Statement | IR Conversion Strategy |
|---------------|------------------------|
| Variable Declaration (`let`) | Allocate space (if needed) and store initial value |
| Variable Declaration (`var`) | Allocate space and store initial value |
| Assignment | Generate a store instruction (for mutable variables) |
| Block | Process statements in sequence |
| If Statement | Generate conditional branch with basic blocks for then/else |
| While Loop | Generate basic blocks for condition, body, and exit |
| For Loop | Transform to equivalent while loop structure with iterators |
| Return Statement | Generate a return instruction with the computed value |
| Break Statement | Generate a branch to the loop exit block |
| Continue Statement | Generate a branch to the loop condition block |

## 4. SSA Construction

### 4.1 Minimal SSA Algorithm

The compiler constructs SSA form using a variant of the minimal SSA algorithm:

1. Identify all variables and their definitions
2. Build control flow graph (CFG) from AST
3. Calculate dominance frontiers (using Cytron92 algorithm)
4. Place phi nodes at appropriate points
5. Rename variables to ensure single assignment

*(⚠️ IR-SSA-PHI-1 in the specification covers exact algorithm choices)*

### 4.2 Handling Variables

For each variable declaration:
1. Create an alloca instruction at function entry (for mutable vars or those in nested contexts)
2. For each assignment, create a new SSA value
3. For each usage, ensure the correct version is referenced
4. At merge points (if/else, loops), insert phi nodes to merge values

## 5. Control Flow Translation

### 5.1 Conditional Constructs

For if/else expressions and statements:

```
// AST:
if condition {
    then_body
} else {
    else_body
}

// IR:
%condition_value = <evaluate condition>
br_cond i1 %condition_value, label %then_block, label %else_block

then_block:
    <then_body instructions>
    br label %merge_block

else_block:
    <else_body instructions>
    br label %merge_block

merge_block:
    <phi nodes for values that differ between branches>
    <continue with next instructions>
```

### 5.2 Loops

For while loops:

```
// AST:
while condition {
    body
}

// IR:
br label %loop_condition

loop_condition:
    %condition_value = <evaluate condition>
    br_cond i1 %condition_value, label %loop_body, label %loop_exit

loop_body:
    <body instructions>
    br label %loop_condition

loop_exit:
    <continue with next instructions>
```

For for-in loops, the IR generation involves:
1. Setup of the iterator
2. Extraction of current value
3. Check for completion
4. Loop body execution
5. Iterator advancement

## 6. Type Handling

### 6.1 Type Checking and Conversion

During IR generation:
1. Track types of all expressions
2. Insert explicit cast instructions where needed
3. Verify type compatibility for operations
4. Generate appropriate typed instructions (e.g., integer vs. floating-point add)

### 6.2 Common Type Conversions

| Source Type | Target Type | IR Conversion |
|-------------|-------------|---------------|
| Int | Float | `%result = cast i64 %int_val to f64` |
| Float | Int | `%result = cast f64 %float_val to i64` |
| Bool | Int | `%result = cast i1 %bool_val to i64` |
| Int | Bool | `%result = ne i64 %int_val, 0` |
| Char | Int | `%result = zext i32 %char_val to i64` |
| Int | Char | `%result = trunc i64 %int_val to i32` |

## 7. Memory Model Translation

### 7.1 Variable Storage

Variables are managed based on their properties:
- **Stack Variables**: Local immutable values often directly represented as SSA values. Local mutable `var` bindings are typically handled via `alloca` for a memory slot, with `load` and `store` operations.
- **Heap Objects**: Complex data structures like `String` and `Vector<T>` (as defined in `STDLIB_CORE_V0.1.md`) are allocated on the heap. Their creation (e.g., string concatenation, `Vector::new` if it allocates, `vector.push` if it reallocates) will be lowered to `call` instructions invoking the appropriate runtime ABI functions (e.g., `ferra_alloc`, `ferra_realloc`, `ferra_string_concat`) specified in `STDLIB_CORE_V0.1.md` and detailed in the backend ABI.
- **Mutable Variables**: Typically allocated on the stack with load/store operations.

### 7.2 Example: String Handling

Strings require special handling:
1. Creation: Allocate memory for the string data
2. Operations: Generate appropriate string manipulation functions
3. Cleanup: Ensure proper deallocation according to ownership rules

## 8. Error Handling Translation

### 8.1 The `?` Operator

The postfix `?` operator for error propagation is translated to:
1. Check if the expression is an error (e.g., `Result::Err` variant)
2. If an error, extract the error value and return it from the function
3. If not an error, extract the success value and continue

```
// AST:
let x = expr?;

// IR (simplified):
%result = call @function()
%is_error = call @Result.is_error(%result)
br_cond i1 %is_error, label %propagate_error, label %continue

propagate_error:
    %error_value = call @Result.get_error(%result)
    %return_error = call @Result.new_error(%error_value)
    ret {appropriate type} %return_error

continue:
    %success_value = call @Result.get_value(%result)
    // continue with %success_value
```

Any errors during lowering (e.g., use before definition, incompatible types not caught earlier) should emit a type/semantic diagnostic via `diagnostics::Reporter` and substitute an `undef` IR value to continue compilation where possible. This same error handling path is used when a `?` operator's operand is an error.

## 9. Function Translation

### 9.1 Function Body

For each function:
1. Create entry block
2. Process parameters
3. Allocate space for local variables as needed
4. Generate IR for function body
5. Ensure all control paths end with a return

### 9.2 Calls and Returns

Function calls are translated to the `call` instruction:
```
%result = call <return_type> @function_name(%arg1, %arg2, ...)
```

Returns are translated to the `ret` instruction:
```
ret <type> %value   // For value-returning functions
ret void            // For functions returning Unit
```

## 10. Special Cases and Optimizations

### 10.1 Constant Expressions

Constant expressions are evaluated at compile-time when possible and represented as constant values in the IR.

### 10.2 Short-Circuit Evaluation

Logical operators (`&&`, `||`) use conditional branching for short-circuit evaluation:

```
// AST: a && b

// IR:
%a_value = <evaluate a>
br_cond i1 %a_value, label %evaluate_b, label %short_circuit

evaluate_b:
    %b_value = <evaluate b>
    br label %merge

short_circuit:
    br label %merge

merge:
    %result = phi i1 [%b_value, %evaluate_b], [false, %short_circuit]
```

## 11. Incremental Compilation Support

- Each AST Item lowers independently; module-level stamp for change detection
- Stable `ValueIdx` numbering per function to minimize downstream differences
- Caching of conversion results to avoid redundant processing of unchanged AST nodes

## 12. Examples

### 12.1 Simple Function

```ferra
fn add(a: Int, b: Int) -> Int {
    return a + b
}
```

Translates to:

```
function @add(i64 %a, i64 %b) -> i64 {
  entry:
    %result = add i64 %a, %b
    ret i64 %result
}
```

### 12.2 Function with Control Flow

```ferra
fn max(a: Int, b: Int) -> Int {
    if a > b {
        return a
    } else {
        return b
    }
}
```

Translates to:

```
function @max(i64 %a, i64 %b) -> i64 {
  entry:
    %cond = gt i64 %a, %b
    br_cond i1 %cond, label %then, label %else

  then:
    ret i64 %a

  else:
    ret i64 %b
}
```

### 12.3 Function with Local Variables

```ferra
fn calculate(x: Int) -> Int {
    let y = x * 2
    let z = y + 10
    return z
}
```

Translates to:

```
function @calculate(i64 %x) -> i64 {
  entry:
    %y = mul i64 %x, 2
    %z = add i64 %y, 10
    ret i64 %z
}
```

## 13. Special Considerations

### 13.1 Debugging Information

The AST to IR conversion also attaches source location and other debugging information, enabling:
- Source-level debugging
- Meaningful error messages
- AI-assisted diagnostics

### 13.2 Performance Considerations

The conversion process aims to generate optimal IR by:
- Minimizing redundant load/store operations
- Eliminating dead code early
- Using SSA properties to enable subsequent optimizations

## 14. Open Questions / TBD

| Tag | Issue |
|-----|-------|
| IR-PAT-1 | Full pattern-matching to IR decision tree strategy |
| IR-LOOP-VAR-1 | Optimized lowering for `for` over arrays vs. iterators |
| IR-EXCEPT-1 | Representation of error-prop `?` at IR (desugar to `br` on null?) |
| IR-MUT-1 | Handling `var` mutable vars that escape closures (may require `alloca`) |
| IR-DBG-SPAN-1 | Mapping of SSA ValueIdx back to source spans for debugger |

## 15. Appendix

### A. AST Node Types Reference

(Complete list of AST node types and their structure to be expanded here)

### B. Conversion Algorithm Pseudocode

(Detailed pseudocode of key conversion algorithms to be expanded here) 