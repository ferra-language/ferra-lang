# Ferra Project Documents Map

This document provides a centralized list of all key design documents, specifications, and planning materials for the Ferra project. It tracks their current status as per the `comprehensive_plan.md`.

## I. Core Project & Planning Documents

| Document                      | Path                               | Status (from comprehensive_plan.md) | Notes                                     |
|-------------------------------|------------------------------------|-------------------------------------|-------------------------------------------|
| Project Overview              | `PROJECT_OVERVIEW.md`              | N/A (Informational)                 | High-level project vision, Git, tools.    |
| Comprehensive Project Plan    | `comprehensive_plan.md`            | N/A (Guiding Document)              | Main project roadmap and phased plan.     |
| Steps (Dr. Hoare's Feedback)  | `Steps.md`                         | N/A (Guiding Document)              | Key technical direction and initial spec. |
| Coding Standards              | `CODING_STANDARDS.md`              | Module 0.1 - Done (✅)             | For compiler/tooling codebase (Rust).     |
| Specification Overview        | `SPEC_OVERVIEW.md`                 | Module 0.3 - Done (✅)             | Explains `lang-spec-v4.yaml`.           |
| `lang-spec-v4.yaml`           | `../lang-spec-v4.yaml`             | Module 0.3 - Done (✅)             | Machine-readable spec summary (in root).  |
| Solidification Checklist      | `SOLIDIFICATION_CHECKLIST.md`      | N/A (Tool)                          | Checklist for language feature stability. |
| Project Documents Map         | `PROJECT_DOCS_MAP.md`              | Continuous Activity                 | This document.                            |

## II. Language Specification Documents

### Phase 0: Foundational Specifications

| Document                         | Path                                     | Status (from comprehensive_plan.md) | Notes                                       |
|----------------------------------|------------------------------------------|-------------------------------------|---------------------------------------------|
| Syntax & Grammar (v0.1)          | `SYNTAX_GRAMMAR_V0.1.md`                 | Module 0.2 - Done (✅)             | EBNF, token definitions, indentation. Updated with FFI syntax (extern blocks, attributes).       |
| Core Semantics (v0.1)            | `CORE_SEMANTICS_V0.1.md`                 | Module 0.2 - Done (✅)             | Execution model, built-in types, evaluation.|
| Ownership Model Principles (v0.1)| `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md`     | Module 0.2 - Done (✅)             | Initial ideas for ownership/borrowing.      |

### Phase 1 & 2: Detailed Design Specifications

| Document                         | Path                                     | Status (from comprehensive_plan.md) | Notes                                       |
|----------------------------------|------------------------------------------|-------------------------------------|---------------------------------------------|
| Lexer Design                     | `DESIGN_LEXER.md`                        | Module 1.1 - In Progress (🔄)      | Ragel-based, Unicode, indentation.          |
| Parser Design                    | `DESIGN_PARSER.md`                       | Module 1.1 - In Progress (🔄)      | Recursive Descent, Pratt, error recovery. Aligned with FFI syntax requirements.   |
| AST Specification                | `AST_SPECIFICATION.md`                   | Module 1.1 - In Progress (🔄)      | AST node structure, arena allocation.       |
| Type Inference Design            | `DESIGN_TYPE_INFERENCE.md`               | Module 1.2 - In Progress (Implicit) | Hindley-Milner, bidirectional, `_`.         |
| Diagnostics Design               | `DESIGN_DIAGNOSTICS.md`                  | Module 1.2 - In Progress (Implicit) | Error reporting, positive-first messages.   |
| IR Specification                 | `IR_SPECIFICATION.md`                    | Module 1.3 - In Progress (Implicit) | Self-Describing SSA IR, quadruples.         |
| AST to IR Conversion             | `AST_TO_IR_CONVERSION.md`                | Module 1.3 - In Progress (Implicit) | Logic for converting AST to IR.             |
| IR Semantic Tags                 | `IR_SEMANTIC_TAGS.md`                    | Module 1.3 - In Progress (Implicit) | `.note.ai` section, CBOR tags.              |
| Standard Library Core (v0.1)     | `STDLIB_CORE_V0.1.md`                    | Module 1.5 - Done (✅)             | I/O, String, Vector, Map (minimal).       |
| AI API - AST Access              | `AI_API_AST.md`                          | Module 1.6 - Done (✅)             | `ai::ast()`, `.note.ai` for tags.         |
| VSCode Plugin (Alpha) Spec       | `VSCODE_PLUGIN_ALPHA_SPEC.md`            | Module 1.7 - Done (✅)             | Syntax highlighting, `lang new` scaffold.   |
| Self-Hosting Subset              | `SELF_HOSTING_SUBSET.md`                 | Module 1.8 - Done (✅)             | Minimal language for compiler tooling.      |
| Concurrency Model                | `CONCURRENCY_MODEL.md`                   | Module 2.1 - Done (✅)             | Deterministic Actors, async/await.          |
| Ownership & Borrow Checker       | `OWNERSHIP_BORROW_CHECKER.md`            | Module 2.2 - Done (✅)             | Refined borrow rules, UX, AI tags.        |
| Backend - WASM & WASI            | `BACKEND_WASM_WASI.md`                   | Module 2.3 - Done (✅)             | WASM generation, WASI, TS bindings.       |
| Package Manager Spec (Beta)      | `PACKAGE_MANAGER_SPEC.md`                | Module 2.4 - Done (✅)             | Content-addressed, CLI, SBOM, Sigstore.   |
| Frontend Enhancements Design     | `FRONTEND_ENHANCEMENTS.md`               | Module 2.5 - Done (✅)             | Row poly, bidirectional, diagnostics.     |
| FFI (Initial C/C++) Design     | `FFI_C_CPP.md`                           | Module 2.6 - Done (✅)             | Interop with C and C++ code.              |
| UI-DSL for Mobile (Preview)    | `UI_DSL_MOBILE.md`                       | Module 3.1 - Done (✅)             | Common UI-DSL for iOS/Android.            |
| UI-DSL Roadmap (Post-Preview)  | `UI_DSL_ROADMAP.md`                      | Module 3.1 (Informational)         | Potential future for UI-DSL.              |
| Energy Profiler Design         | `ENERGY_PROFILER.md`                     | Module 3.2 - Done (✅)             | Design for energy estimation and budget.  |
| AI APIs Refactor & Verify      | `AI_API_REFACTOR_VERIFY.md`              | Module 3.3 - Done (✅)             | Functional design for `ai::refactor` & `ai::verify`. |
| Security Model Design          | `SECURITY_MODEL.md`                      | Module 3.4 - Done (✅)             | Capability-based permissions & sandboxing. |
| Data-Parallel & GPU Support    | `DATA_PARALLEL_GPU.md`                   | Module 3.5 - Done (✅)             | `for_each` loops, CPU SIMD, GPU SPIR-V.    |
| Expanded Backends Design       | `BACKEND_EXPANDED_TARGETS.md`            | Module 3.6 - Done (✅)             | ARM-64, Apple Bitcode, Android AAB.       |
| Diagnostic Codes Registry        | `diagnostic_codes.md`                    | Module 1.2 (Related)                | Central registry for E_xxx, W_xxx codes.  |

### Phase 1 Backends (LLVM)
| Document                         | Path                                     | Status (from comprehensive_plan.md) | Notes                                       |
|----------------------------------|------------------------------------------|-------------------------------------|---------------------------------------------|
| Backend - LLVM (x86-64)          | `BACKEND_LLVM_X86-64.md`                 | Module 1.4 - In Progress (Implicit) | LLVM IR conversion for x86-64.            |

## III. RFCs (Requests for Comments)

*(This section will list formal RFC documents as they are created. Example format below.)*

| RFC Number & Title              | Path                               | Status      | Notes                               |
|---------------------------------|------------------------------------|-------------|-------------------------------------|
| RFC-001: Syntax & Grammar       | `rfc/RFC-001_SYNTAX_GRAMMAR.md`    | Accepted    | Detailed syntax proposal (Phase 0). |
| RFC-002: Core Semantics         | `rfc/RFC-002_CORE_SEMANTICS.md`  | Draft       | (Placeholder for detailed semantics)  |
| ...                             | ...                                | ...         | ...                                 |

## IV. Teaching Materials

| Document                         | Path                                     | Status (from comprehensive_plan.md) | Notes                                       |
|----------------------------------|------------------------------------------|-------------------------------------|---------------------------------------------|
| Teaching Materials Initial    | `TEACHING_MATERIALS_INITIAL.md`    | Module 3.7 - Done (✅)             | Day-0 and Week-1 teaching materials outline. |
| Teaching - Day-0 Lab         | `teaching/day-0/hello_ferra.md`    | Module 3.7 - Done (✅)             | "Hello, Ferra!" lab with video script.      |
| Teaching - Week-1 Tutorial   | `teaching/week-1/rest_api.md`      | Module 3.7 - Done (✅)             | REST API tutorial with video episodes.      |

---
*Status Notes:*
*   "Done (✅)": As marked in `comprehensive_plan.md`.
*   "In Progress (🔄)": Module started, document exists and is being actively developed/refined.
*   "In Progress (Implicit)": Document exists as part of a module that is implicitly in progress based on `comprehensive_plan.md` targets (e.g., Phase 1 documents not explicitly marked done but required for Phase 1 goals).
*   "In Progress (Current)": The module/document currently being actively worked on.
*   "Continuous Activity": Updated as needed throughout the project.
*   "N/A (Informational/Guiding)": Not a deliverable of a specific module step but provides context.

This map should be updated whenever new design documents are created or the status of existing documents changes significantly. 