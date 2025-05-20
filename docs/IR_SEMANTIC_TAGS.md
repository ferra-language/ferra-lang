# IR Semantic Tags & `.note.ai` Section (v0.1)

> **Status:** Draft — Module 1.3 · Step 1.3.3
>
> Defines compact metadata attached to IR artifacts and persisted into object files for AI tooling and IDEs.

---

## 1 Motivation

* Allow **structured provenance** & richer tooling (e.g., `ai::ast()`, `ai::explain(err)`).
* Keep compiler core agnostic; tags are optional CBOR blobs.
* Emit once per compilation; consumers (IDE, CI) can read without full debug info.
* Support versioning independently of IR core features (see TAG‑SCHEMA‑1).

---

## 2 Binary Layout

```
ELF Section : .note.ai
name_size = 4           // "FERA"
desc_size = var         // CBOR map length in bytes
type      = 0xA11E      // vendor‑specific note type
name      = "FERA"      // 4 bytes + NUL pad to 4‑byte alignment
desc      = cbor_map
```

All multi‑byte integers are **little‑endian** for cross‑architecture compatibility.

*For WebAssembly*: custom section `ferra.ai` with the same CBOR payload.

---

## 3 CBOR Map Keys (canonical numeric form)

| Key (int) | Tag Name      | Applies to             | Value Type                | Description                                                              |
| --------- | ------------- | ---------------------- | ------------------------- | ------------------------------------------------------------------------ |
| –1        | `version`     | Module                 | text                      | Tag schema version string (e.g., "0.1").                                 |
| 0         | `source_span` | All IR values & blocks | array `[file_id, lo, hi]` | Precise original span (duplicates IR span for ease).                     |
| 1         | `doc`         | Function / DataClass   | text                      | Docstring markdown.                                                      |
| 2         | `symbol_name` | Function / Global      | text                      | Mangled name after lowering.                                             |
| 3         | `ai_hint`     | Any                    | text                      | Free‑form hint for AI (specific AI tags below are preferred).            |
| ...       | ...           | ...                    | ...                       | (Reserved for future core Ferra tags: 4-63)                              |
| 16        | `owner_func`  | Value                  | uint (FuncIdx)            | For SSA values: back‑reference.                      |
| 17        | `type_idx`    | Value                  | uint (TypeIdx)            | Duplicated to avoid type‑table lookup in AI tools.   |
| 32        | `inline`      | Function               | bool                      | Suggested inline.                                    |
| 33        | `vectorize`   | Loop                   | bool                      | Suggested vectorization opportunity.                 |
| 64        | `ai.assume`   | Any                    | text / map                | Assertion hints for AI analysis (see `AI_API_AST.md`).                   |
| 65        | `ai.explain`  | Item/Value             | text                      | Human‐readable explanation strings (see `AI_API_AST.md`).                |
| 66        | `ai.verify`   | Any                    | text / map                | Verification conditions (e.g., invariants, see `AI_API_AST.md`).         |
| 67        | `provenance.ai`| Module, Item, Block    | map                       | Stores X-AI-Provenance data (see `AI_API_REFACTOR_VERIFY.md` §4).        |
| 68        | `target.gpu_kernel` | Function, Loop         | map (or bool)             | Marks item for SPIR-V compilation (see `DATA_PARALLEL_GPU.md` §4.7).     |
| ...       | ...           | ...                    | ...                       | (Reserved for future AI/tooling tags: 69-95)                             |
| ...       | ...           | ...                    | ...                       | (Reserved for user/custom tags: 96-255)                                  |

(⚠️ **TAG‑ALLOC‑10xx** reserved for third‑party tools.)
(⚠️ **64‑95** reserved for future Ferra core expansions.)

---

## 4 Human‑Readable Tag Examples

> Binary uses numeric keys above; examples below show pretty JSON for docs & IDE tooling. The `id` field mirrors human‑readable IR names (`%` = value, `@` = function) and is **not** stored in binary.

### 4.1 Source Mapping

```json
{
  "id": "%var_1",
  "kind": "source_loc",
  "data": { "file": "main.ferra", "line": 42, "column": 10, "span": 5 }
}
```

### 4.2 Type Information

```json
{
  "id": "%result",
  "kind": "type_info",
  "data": { "original_type": "Result<User, ApiError>", "generic_args": ["User","ApiError"], "user_defined": true }
}
```

### 4.3 Semantic Intent

```json
{
  "id": "@validate_input",
  "kind": "intent",
  "data": { "category": "validation", "purity": "pure", "side_effects": ["error_reporting"] }
}
```

### 4.4 Optimization Hints

```json
{
  "id": "%loop_1",
  "kind": "opt_hint",
  "data": { "parallelizable": true, "vectorizable": true, "trip_count": "dynamic", "memory_access": "sequential" }
}
```

### 4.5 AI Assumption

```json
{
  "id": "%ptr_1",
  "kind": "ai_assume",
  "data": { "nll": "noalias", "bounds": "checked", "certainty": 0.95 }
}
```

---

## 5 Compiler Tag Lifecycle

1. **Generation**  – AST→IR lowering attaches `source_span`, `type_idx`; macro expansion may add custom tags.
2. **Augmentation** – Mid‑end passes insert optimisation hints (`inline`, `vectorize`).
3. **Propagation** – Tags flow through transforms unless invalidated (e.g., after inlining spans merge).
4. **Serialisation** – On object write, module‑level CBOR map is compressed (zstd if > 16 KiB), optionally signed (`--sign‑ai`), and emitted to `.note.ai` / `ferra.ai`.
5. **Stripping** – `--strip‑ai` flag removes sensitive fields (e.g., source snippets) for release builds.

---

## 6 Access APIs

### 6.1 Compiler helper

```rust
fn get_tag<V>(value: ValueIdx, key: TagKey) -> Option<V> { … }
```

### 6.2 Runtime helper

```ferra
let tags = ai::tags(ai::current_function());
if tags.has("parallelizable") { … }
```

### 6.3 AI helper

```ferra
let full_ast = ai::ast();
```

---

## 7 Security & Size

* Tags compressed with zstd when section > 16 KiB.
* `--strip‑ai` drops content for confidential builds.
* `--sign‑ai` appends Ed25519 signature (TAG‑AUTH‑1).

---

## 8 Example Walk‑through

```ferra
fn calculate_average(values: [Int]) -> Float {
    if values.is_empty() {
        return 0.0
    }
    let sum   = values.sum()
    let count = values.len() as Float
    return sum as Float / count
}
```

Generated pretty JSON tags:

```json
[
  { "id": "@calculate_average", "kind": "intent", "data": { "category": "math", "purity": "pure" }},
  { "id": "%if_1", "kind": "source_loc", "data": { "file": "stats.ferra", "line": 2, "column": 5, "span": 35 } },
  { "id": "%values_sum", "kind": "opt_hint", "data": { "parallelizable": true, "vectorizable": true } },
  { "id": "%cast_1", "kind": "type_info", "data": { "original_type": "Int", "target_type": "Float", "safety": "lossless" } }
]
```

---

## 9 Open Questions / TBD

| Tag              | Issue                                                         |
| ---------------- | ------------------------------------------------------------- |
| TAG‑SCHEMA‑1     | Versioning strategy as keys evolve (semver vs. tag high‑bit). |
| TAG‑AUTH‑1       | Signature of `.note.ai` block for tamper detection.           |
| TAG‑LSP‑1        | Mapping tags onto LSP Semantic Tokens for IDEs.               |
| TAG‑MACRO‑1      | Representing spans for macro‑generated code.                  |
| TAG‑DIAG‑1       | Integration with diagnostic system for enriched messages.     |
| TAG‑SIZE‑LIMIT‑1 | Per‑tag size caps to prevent excessive bloat.                 |

---

## 10 CBOR Schema Reference

### 10.1 Core Tag Schema (pretty form)

```
Tag = {
  id: string,             // IR entity identifier (debug‑only)
  kind: string,           // Tag category
  data: map,              // Tag‑specific data
  ?source: SourceLoc,
  ?metadata: map
}

SourceLoc = { file: string, line: uint, column: uint, ?span: uint }
```

*Readers must ignore unknown fields for forward‑compatibility.*

### 10.2 Common Data Schemas

```
TypeInfoData  = { original_type: string, ?generic_args: [string], ?user_defined: bool, ?implementation: string }
IntentData    = { ?category: string, ?purity: string, ?side_effects: [string], ?safety: string }
OptHintData   = { ?parallelizable: bool, ?vectorizable: bool, ?trip_count: string, ?memory_access: string, ?priority: uint }
```

---

## Appendix A  Human‑Readable ↔ Numeric Key Mapping

| Text key     | Num key |
| ------------ | ------- |
| version      | –1      |
| source\_span | 0       |
| doc          | 1       |
| symbol\_name | 2       |
| ai\_hint     | 3       |
| owner\_func  | 16      |
| type\_idx    | 17      |
| inline       | 32      |
| vectorize    | 33      |

---

## Appendix B  CBOR Technical Details

> TBD: canonical ordering, indefinite‑length encoding rules, floating‑point endian conversion. 

### Proposed/UI-Related Tags

*   **Tag**: `ferra::ui_state_variable` (or `ai::tag(ui_state_variable)`)
    *   **Applies to**: Variable declarations (`let` or `var`) within functions identified as UI components.
    *   **Value**: Typically none (presence implies true), or potentially a CBOR map for future configuration options related to state observation (e.g., `{ "debounce_ms": 16 }`). For v0.1, presence is key.
    *   **Purpose**: Marks a variable whose value changes are intended to trigger a re-composition or update of the UI defined by the component function it resides in. This tag helps the compiler/runtime identify reactive state variables for special handling by the UI framework bridge.
    *   **Notes**:
        *   This tag is primarily proposed for use by the Ferra UI-DSL (see `UI_DSL_MOBILE.md`, Section 2.3).
        *   Its necessity and exact semantics depend on the final implementation choices for state reactivity in the UI-DSL (related to TBD UI-DSL-2 in `UI_DSL_MOBILE.md`).
        *   It aids in making the connection between Ferra's core language and the UI framework's reactive update cycle more explicit at the IR level if direct AST analysis of attributes like `#[State]` is not the chosen implementation path for reactivity. 

### Proposed/AI Tooling & Provenance Tags

*   **Tag**: `provenance.ai` (Numeric Key: **67**)
    *   **Applies to**: Module, Function, Block, or specific Items/Instructions that have been generated or modified by an AI tool.
    *   **Value**: A CBOR map containing structured `X-AI-Provenance` data. The schema for this map is detailed in `AI_API_REFACTOR_VERIFY.md` (Section 4.2) and includes tool name, version, timestamp, input parameters, code hashes, and a signature.
    *   **Purpose**: Stores cryptographically signed provenance information for code generated or modified by AI tools like `ai::refactor`. This enables traceability, accountability, and trust verification.
    *   **Notes**:
        *   This tag is crucial for implementing the `X-AI-Provenance` features specified in `AI_API_REFACTOR_VERIFY.md` (Section 4) and `Steps.md` (Item 4).
        *   The detailed CBOR schema for the value map is TBD (AI-PROV-FORMAT-1 from `AI_API_REFACTOR_VERIFY.md`).
        *   Verification of this provenance data can be done using tools like `lang verify-provenance`. 

*   **Tag**: `target.gpu_kernel` (Numeric Key: **68**; Conceptual name, was `target::gpu_kernel`)
    *   **Applies to**: Functions. Potentially to specific loop constructs in the IR if `#[gpu]` can be applied directly to loops and this needs distinct IR representation.
    *   **Value**: Boolean `true` if no parameters, or a CBOR map specifying GPU target details if the `#[gpu(...)]` attribute supports parameters (e.g., `{ "target_env": "vulkan1.2", "required_capabilities": ["ShaderFloat64", "Int64"] }`).
    *   **Purpose**: Marks an IR function (or relevant IR construct) as intended for compilation to a GPU compute kernel, typically targeting SPIR-V as per `DATA_PARALLEL_GPU.md` (Section 4).
    *   **Notes**:
        *   This tag signals the backend to use the SPIR-V generation pipeline.
        *   The IR validator pass should check that code associated with this tag adheres to the "GPU-Ferra" language subset defined in `DATA_PARALLEL_GPU.md` (Section 4.2).
        *   This replaces TBD DPGPU-IR-REP-1 for key assignment. 