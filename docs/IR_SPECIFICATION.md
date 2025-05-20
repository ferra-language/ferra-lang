# Ferra Intermediate Representation (IR) Specification

## 1. Introduction

This document specifies the Structure and Semantics of Ferra's Self-Describing Static Single Assignment (SSA) Intermediate Representation, which serves as the core internal representation for the Ferra compiler's mid-end.

### 1.1 Purpose

The Ferra IR serves several critical purposes:
- Provides a language-independent representation of Ferra programs after parsing and semantic analysis
- Serves as the foundation for optimization passes
- Acts as input to various backend code generators (LLVM, WebAssembly, etc.)
- Supports the language's AI-native features through semantic tagging

### 1.2 Design Principles

- **Static Single Assignment (SSA)**: Each variable is assigned exactly once, simplifying data flow analysis.
- **Self-Describing**: The IR contains sufficient metadata to support introspection, debugging, and AI tooling.
- **Quadruple-Based**: Uses a quadruple format (operation, destination, operand1, operand2) for clarity and simplicity.
- **Strongly Typed**: Every operation and value has an explicit type.
- **Deterministic Order**: Basic-block list and instruction order are canonical to ease diffing and analysis.
- **Extensible**: Designed to accommodate language evolution and additional target platforms through CBOR extension maps.

## 2. IR Structure Overview

### 2.1 Core Components

Ferra's IR consists of the following core components:

1. **Module**: The top-level container for IR entities
2. **Functions**: Represent procedures/methods with parameters, local variables, and basic blocks
3. **Basic Blocks**: Sequences of instructions with single entry and exit points
4. **Instructions**: Quadruple-format operations that transform or manipulate values
5. **Types**: Representations of Ferra's type system in the IR
6. **Constants**: Literal values embedded in the IR
7. **Metadata**: Additional information attached to IR entities

### 2.2 Naming and Identification

- All IR entities have unique identifiers
- Local values (instruction results) are named with a `%` prefix followed by either:
  - A numeric identifier: `%0`, `%1`, `%2`
  - A descriptive name: `%sum`, `%index`, `%temp`
- Global values (functions, globals) use the `@` prefix: `@main`, `@print_string`
- Basic blocks are labeled with a descriptive name followed by a colon: `entry:`, `loop_body:`, `exit:`

## 3. Value & Index Spaces

The IR uses distinct identifier types to prevent accidental confusion between different kinds of references:

```
newtype ValueIdx(u32)   // SSA value id
newtype BlockIdx(u32)   // Basic block id
newtype InstrIdx(u32)   // Instruction id
newtype TypeIdx(u32)    // Type table entry id
```

Each vector in the module owns a contiguous id space (0-based). No global uniqueness is required across functions, as values are resolved within their containing function scope.

## 4. Detailed IR Components

### 4.1 Module

A module represents a compilation unit and contains:

```
Module {
  name: String,                  // Module name
  source_file: String,           // Original source file
  functions: [Function],         // Collection of functions
  globals: [GlobalVariable],     // Global variables
  types: [TypeDeclaration],      // User-defined types
  imports: [Import],             // Imported modules/symbols
  metadata: {String: Metadata}   // Module-level metadata
}
```

### 4.2 Function

A function contains parameters, local variables, and basic blocks:

```
Function {
  name: String,                  // Function name (e.g., @main)
  signature: TypeIdx,            // FnSig in type table
  linkage: LinkageType,          // Public, private, etc.
  parameters: [Parameter],       // Function parameters
  basic_blocks: [BasicBlock],    // Basic blocks of instructions
  cfg_edges: [(BlockIdx, BlockIdx, EdgeKind)], // Explicit CFG for tools
  metadata: {String: Metadata}   // Function metadata
  span: Span                     // Original source range
}
```

### 4.3 Basic Block

A basic block is a sequence of instructions with a single entry and exit point:

```
BasicBlock {
  label: BlockIdx,               // Block label (e.g., "entry:")
  instructions: [InstrIdx],      // Instructions in this block
  predecessors: [BlockIdx],      // Blocks that can flow to this one
  successors: [BlockIdx],        // Blocks this can flow to
  metadata: {String: Metadata}   // Block-level metadata
  span: Span                     // Aggregate of contained AST spans
}
```

### 4.4 Instruction

Instructions are in quadruple format with an extensible extra map:

```
Instruction {
  idx: InstrIdx,                 // Unique instruction identifier
  opcode: Opcode,                // Operation code
  dest: ValueIdx?,               // Destination (if producing a value, none for terminators)
  args: [ValueIdx; 0..2],        // Fixed positional arguments (0, 1, or 2)
  type: TypeIdx,                 // Result type
  extra: {String: Value},        // CBOR map for extensibility (serialized as CBOR in binary format)
  metadata: {String: Metadata}   // Instruction metadata
  span: Span                     // Original source location
}
```

### 4.5 Types

The IR supports all Ferra types:

```
TypeDecl {
  idx: TypeIdx,                  // Numeric handle referenced by instructions
  kind: TypeKind                 // The actual type definition
}

TypeKind = 
  | VoidType                     // Represents Unit type ()
  | IntegerType { width: int }   // Integer types (I8, I16, I32, I64, etc.)
  | FloatType { precision: int } // Float types (F32, F64)
  | BoolType                     // Boolean type
  | CharType                     // Unicode character type (32-bit codepoint)
  | StringType                   // String type (slice { i8*, i64 })
  | ArrayType { element: TypeIdx }  // Array types
  | TupleType { elements: [TypeIdx] } // Tuple types
  | FunctionType {               // Function types
      parameters: [TypeIdx],
      return_type: TypeIdx
    }
  | ReferenceType {              // Reference types
      pointee: TypeIdx,
      mutable: bool
    }
  | UserDefinedType { name: String } // User defined types
```

## 5. Core Instruction Set

### 5.1 Constants and Arithmetic Operations

- **Constants**:
  - `iconst`: Integer constant (`%result = iconst <type> 42`)
  - `fconst`: Float constant (`%result = fconst <type> 3.14`)
  - `boolconst`: Boolean constant (`%result = boolconst <type> true`)
  - `strconst`: String constant (`%result = strconst <type> "Hello, world!"`)
  - `unit`: Unit value (`%result = unit`)

- **Arithmetic**:
  - `add`: Addition (`%result = add <type> %op1, %op2`)
  - `sub`: Subtraction (`%result = sub <type> %op1, %op2`)
  - `mul`: Multiplication (`%result = mul <type> %op1, %op2`)
  - `div`: Division (`%result = div <type> %op1, %op2`)
  - `rem`: Remainder (`%result = rem <type> %op1, %op2`)

### 5.2a Logical Operations (on Booleans)

- `lnot`: Logical NOT (`%result = lnot bool %op1`) (*Complements a boolean value*)
  (* Note: `and`, `or` are typically handled via control flow for short-circuiting, see AST_TO_IR_CONVERSION.md *)

### 5.2 Bitwise Operations

- `and`: Bitwise AND (`%result = and <type> %op1, %op2`)
- `or`: Bitwise OR (`%result = or <type> %op1, %op2`)
- `xor`: Bitwise XOR (`%result = xor <type> %op1, %op2`)
- `shl`: Shift left (`%result = shl <type> %op1, %amount`)
- `shr`: Shift right (`%result = shr <type> %op1, %amount`)

### 5.3 Comparison Operations

- `eq`: Equality (`%result = eq <type> %op1, %op2`)
- `ne`: Inequality (`%result = ne <type> %op1, %op2`)
- `lt`: Less than (`%result = lt <type> %op1, %op2`)
- `le`: Less than or equal (`%result = le <type> %op1, %op2`)
- `gt`: Greater than (`%result = gt <type> %op1, %op2`)
- `ge`: Greater than or equal (`%result = ge <type> %op1, %op2`)

### 5.4 Control Flow

- `br`: Unconditional branch (`br label %target`)
- `br_cond`: Conditional branch (`br_cond i1 %condition, label %true_target, label %false_target`)
- `ret`: Return (`ret <type> %value` or `ret void`)
- `unreachable`: Marks code that can never be executed (for optimization hints)

### 5.5 Memory Operations

- `alloca`: Stack allocation (`%ptr = alloca <type>`)
- `load`: Load from memory (`%value = load <type> %ptr`)
- `store`: Store to memory (`store <type> %value, %ptr`)
- `getfieldptr`: Get a pointer to a field within an aggregate (struct/data) type. 
  (`%field_ptr = getfieldptr <aggregate_type_ptr> %aggregate_ptr, <field_index_or_name>`) 
  (* This instruction is essential for accessing fields of `data` types. The `field_index_or_name` would resolve to a byte offset based on the definition of `aggregate_type_ptr`.*)

### 5.6 Function Operations

- `call`: Function call (`%result = call <return_type> @function(%arg1, %arg2, ...)`)
- `phi`: PHI node for SSA form (`%result = phi <type> [%val1, %block1], [%val2, %block2], ...`)
- `select`: Select between two values based on condition (`%result = select i1 %cond, <type> %ifTrue, <type> %ifFalse`)

### 5.7 Type Operations

- `cast`: Type conversion (`%result = cast <from_type> %value to <to_type>`)

## 6. Example IR

Here's a simple example showing the IR for a function that computes the sum of two integers:

```
function @add_integers(i64 %a, i64 %b) -> i64 {
  entry:
    %result = add i64 %a, %b
    ret i64 %result
}
```

And a more complex example with control flow:

```
function @max(i64 %a, i64 %b) -> i64 {
  entry:
    %cond = gt i64 %a, %b
    br_cond i1 %cond, label %then, label %else

  then:
    br label %exit(%a)

  else:
    br label %exit(%b)

  exit(%result: i64):
    ret i64 %result
}
```

## 7. IR Verification

The IR includes a verification phase that ensures:

1. SSA property is maintained (each variable defined exactly once)
2. Type consistency across all operations
3. Control flow validity (e.g., no unreachable code)
4. Reference safety (validity of loads/stores)

## 8. Serialization Format

The IR can be serialized to/from:

### 8.1 Binary Format

The binary format starts with a self-describing header:

```
FER\0    // Magic number (46 45 52 00)
u16 ver_major = 0
u16 ver_minor = 1
u32 type_count
u32 fn_count
u32 data_count
// Followed by CBOR map of module-level semantic tags (see IR_SEMANTIC_TAGS.md)
// Then type table, functions, and data segments
```

### 8.2 Text Format

Human-readable representation (shown in examples above).

### 8.3 In-Memory Representation

Direct manipulation by the compiler through Rust datastructures.

## 9. Future Extensions

The following extensions are planned for future versions:

1. Advanced control flow for exception handling
2. Additional instructions for concurrency primitives
3. Extended metadata for AI-assisted optimization
4. Target-specific annotations for backend code generators

## 10. Open Questions / TBD

| Tag | Issue |
|-----|-------|
| IR-SSA-PHI-1 | Placement algorithm vs. explicit insertion during AST lower? |
| IR-OPCODES-EXPAND-1 | Complete opcode list including div-mod trapping semantics |
| IR-MEM-1 | Memory model: explicit `load`/`store` vs. all values SSA? Panic ABI: `@ferra_panic(i32 %file_id, i32 %line, i32 %col, i8* %msg)` |
| IR-INLINE-DATA-1 | Representation of large const data blobs |
| IR-BINARY-FORMAT-1 | Decide between custom vs. protobuf for on-disk format |

## 11. Appendix

### A. Complete Instruction Reference

(Detailed reference of all instructions with their precise semantics to be expanded here)

### B. Metadata Keys

(Standard metadata keys and their meaning to be expanded here) 