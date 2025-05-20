# Phase 1 Specification Solidification Checklist

> **Status:** In Progress

## Introduction

This document tracks the verification and refinement of the draft design specifications for Ferra Phase 1 (Modules 1.1 - 1.4). The goal is to ensure these documents are consistent, complete enough for initial implementation, and aligned with already completed specifications.

For each checklist item, the status can be:
*   **To Verify**: Needs review against the relevant document(s).
*   **Pending Action**: Specific changes or additions are required in the document.
*   **Addressed**: The point has been verified and any necessary actions completed.
*   **N/A**: Not applicable.

---

## Module 1.1: Front-End - Lexer & Parser Design

### `docs/DESIGN_LEXER.md`

| #     | Checklist Item                                                                                                | Status     | Action/Notes                                                                                                |
|-------|---------------------------------------------------------------------------------------------------------------|------------|-------------------------------------------------------------------------------------------------------------|
| 1.1.1 | **All Keywords/Operators Listed?**: Explicitly lists all keywords/operators from `SYNTAX_GRAMMAR_V0.1.md`.        | Addressed  | Verified. `DESIGN_LEXER.md` updated to explicitly list current keywords and note future expansion.        |
| 1.1.2 | **INDENT/DENT Rules**: Clear, robust rules for `INDENT`/`DEDENT` emission (handles empty lines, comments).    | Addressed  | Verified. Tab policy (LEX-1) resolved and added to `DESIGN_LEXER.md`: Tabs expand to 4 spaces; mixed tabs/spaces in indent is an error. Rules for emission are otherwise clear. |
| 1.1.3 | **Newline Suppression**: Clarifies handling of "incomplete line" newline suppression (e.g., raw `NEWLINE`s).    | Addressed  | Verified. Approach is consistent: lexer emits `NEWLINE`, parser decides termination. Reference updated in `DESIGN_LEXER.md`. |
| 1.1.4 | **Token Payload**: Specifies exact payload of tokens (type, lexeme, source span).                             | Addressed  | Verified. Section 10 of `DESIGN_LEXER.md` clearly defines token payload (type, lexeme, value, source location). |
| 1.1.5 | **Char Literal Lexing**: `Char` literal lexing rule (`'x'`) present and correct.                                | Addressed  | Verified. `DESIGN_LEXER.md` updated in Sec 2, 7, 11 to include full Character Literal support, aligning with grammar and self-hosting subset. |

### `docs/DESIGN_PARSER.md`

| #     | Checklist Item                                                                                                | Status     | Action/Notes                                                                                             |
|-------|---------------------------------------------------------------------------------------------------------------|------------|----------------------------------------------------------------------------------------------------------|
| 1.1.6 | **Pratt Parser Precedence**: Correctly reflects operator precedence/associativity from `SYNTAX_GRAMMAR_V0.1.md`. | Addressed  | Verified. `DESIGN_PARSER.md` updated to explicitly state that precedence/associativity is derived from `SYNTAX_GRAMMAR_V0.1.md`, Appendix A. |
| 1.1.7 | **GLR Fallback / Indentation Strategy**: Clarifies GLR fallback use or confirms lexer-driven sufficiency.        | Addressed  | Verified. `DESIGN_PARSER.md` (Sec 2 & 7) clarifies lexer-driven INDENT/DEDENT is primary for indentation; GLR is a general fallback for other complex ambiguities. |
| 1.1.8 | **Parsing Rules Completeness**: Clear parsing rules for all v0.1 grammar constructs.                          | Addressed  | Verified. `DESIGN_PARSER.md` generally covers grammar. Details for Pattern parsing within MatchExpr added for further clarity. |
| 1.1.9 | **Error Recovery**: Describes parser error recovery strategy.                                                 | Addressed  | Verified. `DESIGN_PARSER.md` (Sec 8) updated to explicitly define panic mode with synchronizing tokens as the v0.1 strategy. TBD (PARSE-1) updated. |

### `docs/AST_SPECIFICATION.md`

| #      | Checklist Item                                                                                               | Status     | Action/Notes                                                                                             |
|--------|--------------------------------------------------------------------------------------------------------------|------------|----------------------------------------------------------------------------------------------------------|
| 1.1.10 | **AST Node Coverage**: Every grammar rule in `SYNTAX_GRAMMAR_V0.1.md` has a corresponding AST node definition. | Addressed  | Verified. Coverage is very good. Clarifications for `QualifiedIdentifier` in expressions (`ExprKind::Path`) and `IfStatement` (`StmtKind::If`) added. |
| 1.1.11 | **Node Field Typing**: AST node fields clearly typed (refs to other nodes, literal values, etc.).              | Addressed  | Verified. Field typing in `AST_SPECIFICATION.md` is clear and uses ID-based arena referencing effectively. |
| 1.1.12 | **Source Spans**: Source span info consistently part of every relevant AST node.                               | Addressed  | Verified. `Span` field is consistently included in all relevant AST node structs in `AST_SPECIFICATION.md`. |
| 1.1.13 | **Identifier/Scope Representation**: Clarifies representation at AST stage or deferral.                        | Addressed  | Verified. `AST_SPECIFICATION.md` (Sec 1) updated to clarify AST handles identifier occurrences; full scope/symbol resolution is a semantic analysis task. |
| 1.1.14 | **Stdlib Collection AST Nodes**: Confirms `Vector`/`Map` handled by general mechanisms (not unique AST nodes).   | Addressed  | Verified. `AST_SPECIFICATION.md` appropriately has collection literals as TBD (AST-LIT), consistent with current grammar (no literal syntax) and stdlib focus. |

---

## Module 1.2: Front-End - Type Inference & Basic Diagnostics Design

### `docs/DESIGN_TYPE_INFERENCE.md`

| #     | Checklist Item                                                                                                | Status     | Action/Notes                                                                                             |
|-------|---------------------------------------------------------------------------------------------------------------|------------|----------------------------------------------------------------------------------------------------------|
| 1.2.1 | **Generics Inference**: Details handling of generic functions/data structures (especially from stdlib).         | Addressed  | Verified. `DESIGN_TYPE_INFERENCE.md` (Sec 2 & 3) details HM type schemes/instantiation, covering stdlib generics like Result/Option/Vector. |
| 1.2.2 | **Coercion/Subtyping**: Defines precise rules for type coercion or subtyping (if any for v0.1).                 | Addressed  | Verified. `DESIGN_TYPE_INFERENCE.md` (Sec 1) states no implicit numeric coercion for v0.1; row polymorphism (structural subtyping) is TBD (Sec 6, TYPE-ROWPOLY-1). |
| 1.2.3 | **Type Error Reporting**: Details how errors like "type mismatch" or "occurs check" are handled/reported.       | Addressed  | Verified. `DESIGN_TYPE_INFERENCE.md` (Sec 7) clearly outlines reporting requirements for these HM errors. |
| 1.2.4 | **Annotation/Inference Conflicts**: Defines resolution/reporting for conflicts.                                 | Addressed  | Verified. `DESIGN_TYPE_INFERENCE.md` (Sec 4) implies conflicts (e.g., annotation vs. initializer) result in unification errors (type mismatches). |
| 1.2.5 | **`_` Placeholder Semantics**: Defines specific behavior of `_` type placeholder.                               | Addressed  | Verified. `DESIGN_TYPE_INFERENCE.md` (Sec 3) clearly states `_` introduces a fresh type variable to be inferred from context. |

### `docs/DESIGN_DIAGNOSTICS.md`

| #     | Checklist Item                                                                                                | Status     | Action/Notes                                                                                               |
|-------|---------------------------------------------------------------------------------------------------------------|------------|------------------------------------------------------------------------------------------------------------|
| 1.2.6 | **JSON Format Alignment**: Precisely defined and 100% matches `VSCODE_PLUGIN_ALPHA_SPEC.md` JSON format.      | Addressed  | Verified. `DESIGN_DIAGNOSTICS.md` updated (new §3.2, updated §9) to specify JSON line protocol for tooling, aligning with `VSCODE_PLUGIN_ALPHA_SPEC.md` (fields: severity, message, file_path, span {lo,hi}, code, help). |
| 1.2.7 | **Multiple Error Reporting**: Basic strategies for reporting multiple errors in v0.1.                         | Addressed  | Verified. `DESIGN_DIAGNOSTICS.md` (Sec 8) outlines basic strategies (per-line limit heuristic, global error cap). |
| 1.2.8 | **Error Message Guidelines**: Includes guidelines for good error messages (clarity, positive-first).          | Addressed  | Verified. `DESIGN_DIAGNOSTICS.md` (Sec 2) provides comprehensive guiding principles for messages.        |
| 1.2.9 | **Source Span Utilization**: Describes best practices for using source spans in diagnostics.                    | Addressed  | Verified. `DESIGN_DIAGNOSTICS.md` (Sec 3 & 4) details span structure and rendering for diagnostics.        |

---

## Module 1.3: Mid-End - SSA IR Design (Initial)

### `docs/IR_SPECIFICATION.md`

| #     | Checklist Item                                                                                                | Status     | Action/Notes                                                                                                 |
|-------|---------------------------------------------------------------------------------------------------------------|------------|--------------------------------------------------------------------------------------------------------------|
| 1.3.1 | **Instruction Set Coverage**: IR covers all ops for `SELF_HOSTING_SUBSET.md` (mem alloc, calls, etc.).      | Addressed  | Verified. `IR_SPECIFICATION.md` updated to include `lnot` (logical not) and `getfieldptr` (field access), covering key needs for self-hosting subset. |
| 1.3.2 | **Type Representation in IR**: Details how Ferra types are represented in IR.                                 | Addressed  | Verified. `IR_SPECIFICATION.md` (Sec 4.5) provides a comprehensive list of `TypeKind` covering Ferra types. |
| 1.3.3 | **SSA Form Construction**: Details SSA construction and maintenance (phi nodes, etc.).                        | Addressed  | Verified. `IR_SPECIFICATION.md` (Sec 2.1, 7) and `AST_TO_IR_CONVERSION.md` (Sec 4) confirm SSA and `phi` usage; TBD `IR-SSA-PHI-1` is for specific algo choice. |
| 1.3.4 | **IR Format**: Defines textual or binary format for the self-describing IR.                                   | Addressed  | Verified. `IR_SPECIFICATION.md` (Sec 8) defines existence and general nature of text and binary formats; TBD `IR-BINARY-FORMAT-1` for encoding details. |
| 1.3.5 | **High-Level Construct Lowering (to IR)**: Details lowering of `match`, `for` loops, etc.                     | Addressed  | Verified. `AST_TO_IR_CONVERSION.md` (not `IR_SPECIFICATION.md`) details these strategies (e.g., Sec 3, 5, 10.2). TBDs like `IR-PAT-1` track full detail for complex cases. |

### `docs/AST_TO_IR_CONVERSION.md`

| #     | Checklist Item                                                                                                | Status     | Action/Notes                                                                                                 |
|-------|---------------------------------------------------------------------------------------------------------------|------------|--------------------------------------------------------------------------------------------------------------|
| 1.3.6 | **AST Node Conversion Rules**: Clear rules for converting each AST node from `SELF_HOSTING_SUBSET.md`.          | Addressed  | Verified. `AST_TO_IR_CONVERSION.md` (Sec 3) has good general coverage. Updated for `FieldAccess` (using `getfieldptr`) and Unary `!` (using `lnot`). TBDs track full detail for some complex constructs. |
| 1.3.7 | **Scope/Variable Handling (IR Gen)**: Details handling during IR generation.                                  | Addressed  | Verified. `AST_TO_IR_CONVERSION.md` (Sec 2, 4.2) details use of `ScopeStack` and SSA variable handling. |
| 1.3.8 | **Memory Allocation in IR**: Represents mem alloc for `String`/`Vector` (e.g., `ferra_alloc` calls).          | Addressed  | Verified. `AST_TO_IR_CONVERSION.md` (Sec 7.1) updated to clarify heap allocations via `call` to runtime ABI functions. |
| 1.3.9 | **Function Call Translation (to IR)**: Details translation of calls and argument passing.                     | Addressed  | Verified. `AST_TO_IR_CONVERSION.md` (Sec 9.2) clearly states use of `call` instruction. |
| 1.3.10| **Error Handling Lowering (to IR)**: Details lowering of `?`, `match` on `Result`.                              | Addressed  | Verified. `AST_TO_IR_CONVERSION.md` (Sec 8.1, 3.2) outlines strategies; TBDs (`IR-EXCEPT-1`, `IR-PAT-1`) track full detail. |

---

## Module 1.4: Back-End - Initial Target Design (e.g., x86-64 via LLVM)

### `docs/BACKEND_LLVM_X86-64.md`

| #     | Checklist Item                                                                                               | Status     | Action/Notes                                                                                             |
|-------|--------------------------------------------------------------------------------------------------------------|------------|----------------------------------------------------------------------------------------------------------|
| 1.4.1 | **Type Mapping (LLVM)**: Details mapping of Ferra types (`String`, `Vector`, `data`) to LLVM types.            | Addressed  | Verified. `BACKEND_LLVM_X86-64.md` (Sec 3.1) updated to clarify Array/Vector and UserDefinedType (data class) mapping to LLVM types. |
| 1.4.2 | **Call Conventions (LLVM)**: Details Ferra call/argument passing conventions in LLVM IR.                       | Addressed  | Verified. `BACKEND_LLVM_X86-64.md` (Sec 3.4) specifies standard target ABIs (SystemV, Microsoft x64), which is sufficient. |
| 1.4.3 | **Error Handling (LLVM)**: Strategy for mapping Ferra error handling (`Result`, `?`) to LLVM IR.               | Addressed  | Verified. `BACKEND_LLVM_X86-64.md` (Sec 3.1, 3.9) details `Result<T,E>` ABI and `?` operator lowering to LLVM IR. |
| 1.4.4 | **Runtime Calls (LLVM)**: Details handling of runtime mem mgt calls (`ferra_alloc` from stdlib).               | Addressed  | Verified. `BACKEND_LLVM_X86-64.md` (Sec 3.3, 4.1, TBD BACKEND-RT-1) lists runtime ABI. Name `ferra_dealloc` changed to `ferra_free` for consistency. |
| 1.4.5 | **Optimization Passes (LLVM)**: Lists default LLVM optimization passes planned.                                | Addressed  | Verified. `BACKEND_LLVM_X86-64.md` (Sec 5) outlines optimization pass strategy and level mapping. |
| 1.4.6 | **Debug Info (LLVM)**: Outlines approach for generating DWARF/CodeView via LLVM.                               | Addressed  | Verified. `BACKEND_LLVM_X86-64.md` (Sec 6) outlines DWARF/PDB generation approach. |
| 1.4.7 | **`.note.ai` Emission (LLVM)**: Details emission of `.note.ai` section into object files via LLVM.             | Addressed  | Verified. `BACKEND_LLVM_X86-64.md` (Sec 3.8, 4.2) updated to clarify embedding the full CBOR semantic tag map into the custom section. |