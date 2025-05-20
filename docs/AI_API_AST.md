# Ferra AI API - AST Access & Semantic Tags v0.1

> **Status:** Initial Draft - Module 1.6 · Steps 1.6.1 & 1.6.2

## 1. Introduction

This document specifies the initial AI-focused APIs for Ferra v0.1, focusing on AST access and semantic tagging. The goal is to enable AI tools to understand and interact with Ferra code at a semantic level, while maintaining a clean separation between the language and AI capabilities.

## 2. Core Concepts

### 2.1. AST Access via `ai::ast()`

The `ai::ast()` function provides programmatic access to the Abstract Syntax Tree (AST) of the current compilation unit.

```ferra
// Conceptual API (exact syntax TBD)
let ast = ai::ast() // Returns UTF-8 JSON string containing AST representation
let ast_json = ast.parse_json();  // compiler intrinsics for ergonomic use
```

*   **Return Type**: `String` (UTF-8 encoded JSON representation of the AST)
*   **Size Limit**: Output truncated after 10 MiB unless `--ai-ast=unbounded` is specified
*   **Availability**: Only available during compilation, not at runtime
*   **Access Control**: 
    *   Requires explicit opt-in via `#![allow(ai_ast)]` at the crate root
    *   Can also be enabled via `--ai-ast` compiler flag for build scripts
    *   Release builds strip tags unless `--ai-ast --keep-ai` combo is passed
    *   _Future Consideration_: This access might be further governed by a formal `ai:access_ast` permission within the capability-based security model (see `SECURITY_MODEL.md` §5.2).
*   **Use Cases**:
    *   AI-powered code analysis
    *   Semantic understanding for refactoring
    *   Documentation generation
    *   Test case generation

### 2.2. Semantic Tags via `.note.ai` Sections

Semantic tags provide additional metadata about code elements, stored in a dedicated section of the compiled binary.

*   **Storage Format**: CBOR (Concise Binary Object Representation)
*   **Section Name**: `.note.ai`
*   **Location**: ELF/wasm section in the compiled binary
*   **Purpose**: Store non-executable metadata for AI tools

## 3. AST Representation

### 3.1. JSON Schema (v0.1)

The AST is exposed as a JSON structure with the following key elements. Note that semantic tags are duplicated here for convenience, but the canonical source is the `.note.ai` section (see §4.2). The JSON representation exists only during compilation for tooling use and is not embedded in the binary.

```json
{
  "version": "0.1",
  "source": {
    "path": "string",
    "content": "string"
  },
  "ast": {
    "kind": "Module",
    "id": 1,  // Numeric node ID matching AST_SPEC
    "items": [
      {
        "kind": "Function",
        "id": 2,  // Numeric node ID
        "name": "string",
        "params": [
          {
            "name": "string",
            "type": "string"
          }
        ],
        "return_type": "string",
        "body": {
          "kind": "Block",
          "id": 3,  // Numeric node ID
          "statements": []
        }
      }
    ]
  },
  "semantic_tags": {
    "tags": [
      {
        "id": "tag1",
        "type": "ai.assume",
        "key": 64,  // Numeric key matching IR_SEMANTIC_TAGS.md table (64-95 range)
        "span": {
          "lo": 100,  // Matches IR span [lo,hi] convention
          "hi": 150
        },
        "data": {}
      }
    ]
  }
}
```

### 3.2. Key AST Nodes

*   **Module**: Root node containing all top-level items
*   **Function**: Function definitions with parameters and body
*   **Data**: Data type definitions
*   **Let/Var**: Variable declarations
*   **Block**: Statement blocks
*   **Expression**: Various expression types (binary, unary, etc.)
*   **Pattern**: Pattern matching constructs

## 4. Semantic Tags

### 4.1. Tag Types

*   **`ai.assume`**: Assertions about code properties
    ```ferra
    // Example: Assert no aliasing in function
    #[ai.assume(nll="noalias")]  // Maps to key 64 in IR_SEMANTIC_TAGS.md
    fn process(data: &mut Data) { ... }
    ```

*   **`ai.explain`**: Documentation for AI tools
    ```ferra
    // Example: Explain complex algorithm
    #[ai.explain("Implements quicksort with median-of-three pivot selection")]  // Maps to key 65
    fn quicksort(arr: &mut [Int]) { ... }
    ```

*   **`ai.verify`**: Verification conditions
    ```ferra
    // Example: Specify loop invariant
    #[ai.verify(invariant="i >= 0 && i < arr.len()")]  // Maps to key 66
    for i in 0..arr.len() { ... }
    ```
    *Note: These attributes declare verification conditions directly in source code. They can serve as input to the `ai::verify()` API suite (detailed in `AI_API_REFACTOR_VERIFY.md`), which performs various verification tasks.*

### 4.2. Tag Storage

Tags are stored in the `.note.ai` section as CBOR maps. The numeric keys map to specific tag types defined in `IR_SEMANTIC_TAGS.md`:

| Key | Tag Type    | Example Usage                                    | Defined in `IR_SEMANTIC_TAGS.md`? |
|-----|-------------|--------------------------------------------------|-----------------------------------|
| 64  | ai.assume   | `#[ai.assume(nll="noalias")]`                    | Yes (Key 64)                      |
| 65  | ai.explain  | `#[ai.explain("Algorithm explanation")]`         | Yes (Key 65)                      |
| 66  | ai.verify   | `#[ai.verify(invariant="i >= 0")]`              | Yes (Key 66)                      |

```cbor
{
  "version": "0.1",
  "tags": [
    {
      "id": "tag1",
      "type": "ai.assume",
      "key": 64,
      "span": [100, 150],  // [lo, hi] format
      "data": {
        "nll": "noalias"
      }
    }
  ]
}
```

## 5. Implementation Notes

### 5.1. Compiler Integration

*   AST access is implemented in the compiler's semantic analysis phase
*   Tags are collected during parsing and stored in the IR
*   The `.note.ai` section is generated during code generation

### 5.2. Security Considerations

*   AST access is disabled by default
*   Tags are stripped from release builds unless explicitly enabled via `--ai-ast --keep-ai`.
*   The JSON returned by `ai::ast()` is transient, exists only during compilation for tooling use, and is never embedded into the final binary.
*   No runtime overhead for tag storage in the final binary.
*   Optional `--sign-ai` flag for cryptographic signing of the `.note.ai` section (this does not apply to the transient JSON AST).

## 6. Open Questions / TBD

| Tag             | Issue                                                                                             |
|-----------------|---------------------------------------------------------------------------------------------------|
| AI-API-1        | Should `ai::ast()` be available at runtime for reflection?                                        |
| AI-API-2        | How to handle AST access in incremental compilation?                                              |
| AI-API-3        | Should we support custom tag types beyond the built-in ones?                                      |
| AI-API-4        | How to version the AST representation and tag format?                                             |
| AI-API-5        | Should we support AST modification for code generation?                                           |
| AI-API-6        | How to map AST node IDs back to source spans for LSP hover?                                       |
| AI-API-7        | Should we support streaming mode for large ASTs?                                                  |
| AI-API-8        | What is the policy for bumping the JSON schema `version` field?                                   |

## 7. Future Directions

*   Integration with AI-powered refactoring tools
*   Support for semantic search and code understanding
*   Automated test case generation
*   Documentation generation
*   Performance optimization suggestions

---

This specification will evolve based on implementation experience and community feedback. The focus for v0.1 is on establishing the basic infrastructure for AI tooling integration. 