# Comprehensive Project Plan: Ferra

This document outlines the comprehensive, phased, and modular plan for designing and specifying Ferra (formerly AI-Native Programming Language). It is based on the initial "Design Brief v4" and heavily incorporates the technical direction and roadmap from Dr. Hoare's feedback in `Steps.md`.

**Core Principle**: Documentation is integral to every phase and module. Each step will conclude with the creation or updating of relevant design documents, specifications, and explanatory materials.

---

## Phase 0: Project Setup & Foundational Decisions (Immediate)

**Goal**: Establish project infrastructure, define initial core language concepts, and publish baseline specifications.

*   **Module 0.1: Project Infrastructure**
    *   **Step 0.1.1**: Set up version control (e.g., Git repository).
        *   *Documentation*: Note on repository location and access.
    *   **Step 0.1.2**: Establish project management tools (e.g., issue tracker, shared document space).
        *   *Documentation*: Links and usage guidelines for project tools.
    *   **Step 0.1.3**: Define initial coding standards (e.g., for compiler development, likely Rust).
        *   *Documentation*: `CODING_STANDARDS.md` for the compiler/tooling codebase.

*   **Module 0.2: Formalize Core Syntax & Semantics (Initial Draft)**
    *   **Step 0.2.1**: Draft initial grammar rules (BNF/EBNF) based on `Steps.md` (Pratt for expressions, GLR fallback for optional significant-indent) and Brief v4 syntax examples.
        *   *Documentation*: `SYNTAX_GRAMMAR_V0.1.md`.
    *   **Step 0.2.2**: Define core data types, mutability (`let` vs `var`), basic control flow, function definitions.
        *   *Documentation*: `CORE_SEMANTICS_V0.1.md`.
    *   **Step 0.2.3**: Outline initial principles for the ownership/borrowing model, emphasizing "positive-first" error messaging.
        *   *Documentation*: `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md`.

*   **Module 0.3: Publish Initial Specification & RFCs**
    *   **Step 0.3.1**: Formally publish the `lang-spec-v4.yaml` header (from `Steps.md`) as the canonical machine-readable specification in the project root.
        *   *Documentation*: `lang-spec-v4.yaml` (content from `Steps.md`), plus a `SPEC_OVERVIEW.md` explaining its purpose and structure.
    *   **Step 0.3.2**: Prepare and open the first RFC (Request for Comments) for the detailed syntax and grammar, inviting feedback.
        *   *Documentation*: `RFC-001_SYNTAX_GRAMMAR.md`.

---

## Phase 1: MVP Compiler & Tooling (Target: Q3 2025)

**Primary Goal**: Design specifications for an MVP compiler (self-hosting subset), and an alpha VSCode plugin.

*   **Module 1.1: Front-End - Lexer & Parser Design**
    *   **Step 1.1.1**: Specify Lexer (Ragel-generated DFA approach, Unicode ID_Start/ID_Continue).
        *   *Documentation*: `DESIGN_LEXER.md`.
    *   **Step 1.1.2**: Specify Parser (Pratt for expressions, GLR fallback, handling optional indentation).
        *   *Documentation*: `DESIGN_PARSER.md`.
    *   **Step 1.1.3**: Define the Abstract Syntax Tree (AST) structure in detail.
        *   *Documentation*: `AST_SPECIFICATION.md`.

*   **Module 1.2: Front-End - Type Inference & Basic Diagnostics Design**
    *   **Step 1.2.1**: Specify Hindley-Milner based type inference (core types, `_` for gradual typing).
        *   *Documentation*: `DESIGN_TYPE_INFERENCE.md`.
    *   **Step 1.2.2**: Design basic diagnostic reporting mechanisms for syntax and type errors.
        *   *Documentation*: `DESIGN_DIAGNOSTICS.md`.

*   **Module 1.3: Mid-End - SSA IR Design (Initial)**
    *   **Step 1.3.1**: Define the structure of the Self-Describing SSA IR (quadruples).
        *   *Documentation*: `IR_SPECIFICATION.md`.
    *   **Step 1.3.2**: Specify AST to IR conversion logic for the initial language subset.
        *   *Documentation*: `AST_TO_IR_CONVERSION.md`.
    *   **Step 1.3.3**: Design the initial mechanism for semantic tags (CBOR maps in `.note.ai` sections).
        *   *Documentation*: `IR_SEMANTIC_TAGS.md`.

*   **Module 1.4: Back-End - Initial Target Design (e.g., x86-64 via LLVM)**
    *   **Step 1.4.1**: Specify LLVM backend infrastructure requirements.
    *   **Step 1.4.2**: Design IR to LLVM IR conversion logic for the current language subset.
        *   *Documentation*: `BACKEND_LLVM_X86-64.md`.

*   **Module 1.5: Core Standard Library (Minimal Set) Design**
    *   **Step 1.5.1**: Specify foundational I/O APIs (e.g., `println`).
    *   **Step 1.5.2**: Specify essential data structures APIs (e.g., String, List/Vector, basic Map/Dictionary).
        *   *Documentation*: `STDLIB_CORE_V0.1.md`.

*   **Module 1.6: AI API - `ai::ast()` Design (Proof of Concept)**
    *   **Step 1.6.1**: Design the `ai::ast()` API to expose the AST (e.g., as JSON or CBOR).
    *   **Step 1.6.2**: Specify the `.note.ai` ELF/wasm section mechanism for storing tags.
        *   *Documentation*: `AI_API_AST.md`.

*   **Module 1.7: VSCode Plugin (Alpha) Design**
    *   **Step 1.7.1**: Specify features: basic syntax highlighting, compiler error integration.
    *   **Step 1.7.2**: Design `lang new myapp` project scaffolding command.
        *   *Documentation*: `VSCODE_PLUGIN_ALPHA_SPEC.md`.

*   **Module 1.8: Self-Hosting Subset Goal Definition**
    *   **Step 1.8.1**: Identify and document the minimal language subset sufficient for writing parts of the compiler/tools.
        *   *Documentation*: `SELF_HOSTING_SUBSET.md`.

---

## Phase 2: Concurrency, WASM, Package Manager Design (Target: Q4 2025)

**Primary Goal**: Design specifications for deterministic async/actors, WASM backend, and a beta package manager.

*   **Module 2.1: Concurrency - Deterministic Actors & Async Design**
    *   **Step 2.1.1**: Design the deterministic actor model (compile-time decided schedule).
    *   **Step 2.1.2**: Specify `async`/`await` syntax and semantics for deterministic execution.
    *   **Step 2.1.3**: Design channels or equivalent message-passing for inter-actor communication.
        *   *Documentation*: `CONCURRENCY_MODEL.md`.

*   **Module 2.2: Borrow Checker & Ownership UX Design (Refinement)**
    *   **Step 2.2.1**: Refine borrow checker design, integrating with actor model and async.
    *   **Step 2.2.2**: Design "positive-first" error messaging for ownership/borrow errors.
    *   **Step 2.2.3**: Specify integration of AI tags like `ai.assume(nll="noalias")`.
        *   *Documentation*: `OWNERSHIP_BORROW_CHECKER.md`.

*   **Module 2.3: Back-End - WebAssembly (WASI) Design**
    *   **Step 2.3.1**: Specify WASM code generation from IR.
    *   **Step 2.3.2**: Ensure WASI compatibility for system interactions.
    *   **Step 2.3.3**: Design profile-guided tree-shaking and lazy stdlib segmentation (target ≤ 200 kB baseline).
    *   **Step 2.3.4**: Design auto-generation of TypeScript bindings.
        *   *Documentation*: `BACKEND_WASM_WASI.md`.

*   **Module 2.4: Package Manager (Beta) Design**
    *   **Step 2.4.1**: Design content-addressed storage (`~/.lang/pkg`).
    *   **Step 2.4.2**: Specify CLI commands: `lang add`, `lang vendor --sbom`.
    *   **Step 2.4.3**: Design SPDX SBOM generation and Sigstore integration for signing.
        *   *Documentation*: `PACKAGE_MANAGER_SPEC.md`.

*   **Module 2.5: Front-End Enhancements Design**
    *   **Step 2.5.1**: Design row polymorphism for records.
    *   **Step 2.5.2**: Refine type inference to be fully bidirectional.
    *   **Step 2.5.3**: Design Bloom-filter de-duplication for diagnostics.
    *   **Step 2.5.4**: Design `ai::explain(err)` integration.
        *   *Documentation*: `FRONTEND_ENHANCEMENTS.md` (updating previous front-end docs).

*   **Module 2.6: FFI (Initial C/C++) Design**
    *   **Step 2.6.1**: Design FFI capabilities for C and C++.
        *   *Documentation*: `FFI_C_CPP.md`.

---

## Phase 3: Mobile/UI, Profiler, Advanced AI Design (Target: Q1 2026)

**Primary Goal**: Design specifications for iOS/Android UI-DSL preview, an energy profiler, and functional AI APIs.

*   **Module 3.1: UI-DSL for iOS/Android (Preview) Design**
    *   **Step 3.1.1**: Design a common UI-DSL.
    *   **Step 3.1.2**: Specify mapping to public SwiftUI/UIKit (iOS).
    *   **Step 3.1.3**: Design `lang doctor ios` static analyzer.
    *   **Step 3.1.4**: Specify mapping to Jetpack Compose (Android, target runtime ≤ 5 MB).
        *   *Documentation*: `UI_DSL_MOBILE.md`.

*   **Module 3.2: Energy Profiler Design**
    *   **Step 3.2.1**: Design LLVM pass (or equivalent) for energy estimation (µops × TDP → joules).
    *   **Step 3.2.2**: Specify integration into test suite (target < 70 J).
    *   **Step 3.2.3**: Design CI checks for energy budget.
        *   *Documentation*: `ENERGY_PROFILER.md`.

*   **Module 3.3: AI APIs - `ai::refactor` & `ai::verify` Design (Functional)**
    *   **Step 3.3.1**: Design `ai::refactor::<goal>()` for common goals.
    *   **Step 3.3.2**: Design `ai::verify()`: re-type-checking, fuzz-testing, coverage enforcement.
    *   **Step 3.3.3**: Specify embedding/verification of `X-AI-Provenance` signatures.
        *   *Documentation*: `AI_API_REFACTOR_VERIFY.md`.

*   **Module 3.4: Security Features Design**
    *   **Step 3.4.1**: Design capability-based permissions (`manifest.perms`).
    *   **Step 3.4.2**: Design sandboxing mechanisms (Wasm+WASI, seccomp-bpf).
        *   *Documentation*: `SECURITY_MODEL.md`.

*   **Module 3.5: Data-Parallel Loops & GPU Support Design**
    *   **Step 3.5.1**: Design data-parallel `for_each` constructs.
    *   **Step 3.5.2**: Specify lowering to CPU SIMD.
    *   **Step 3.5.3**: Design SPIR-V generation for `#[gpu]` attribute.
        *   *Documentation*: `DATA_PARALLEL_GPU.md`.

*   **Module 3.6: Expanded Back-Ends Design**
    *   **Step 3.6.1**: Specify ARM-64 backend.
    *   **Step 3.6.2**: Specify Apple Bitcode backend.
    *   **Step 3.6.3**: Specify Android AAB generation.
        *   *Documentation*: `BACKEND_EXPANDED_TARGETS.md`.

*   **Module 3.7: Teaching Stack - Day-0 & Week-1 Materials Outline**
    *   **Step 3.7.1**: Outline "Day-0 lab" content.
    *   **Step 3.7.2**: Outline "Week-1" tutorial content (e.g., REST API).
    *   **Step 3.7.3**: Define standards for Markdown, narrated videos, subtitles.
        *   *Documentation*: `TEACHING_MATERIALS_INITIAL.md`.

---

## Phase 4: v1.0 Freeze & Governance Design (Target: Q2 2026)

**Primary Goal**: Finalize all design specifications for a v1.0 language and establish governance structures.

*   **Module 4.1: Language & Standard Library Stabilization (v1.0) Design**
    *   **Step 4.1.1**: Finalize language feature set for v1.0.
    *   **Step 4.1.2**: Document comprehensive testing strategies.
    *   **Step 4.1.3**: Finalize and document all standard library domains (IO, net, dataframe, crypto, UI, ML-ops).
        *   *Documentation*: `LANGUAGE_SPEC_V1.0.md`, `STDLIB_SPEC_V1.0.md`.
    *   **Step 4.1.4**: Complete FFI design for Rust, JVM, .NET, Python.
        *   *Documentation*: `FFI_EXPANDED.md`.

*   **Module 4.2: Back-End Completion & Optimization Design**
    *   **Step 4.2.1**: Finalize embedded ELF backend design (< 150 kB, zero-alloc async).
    *   **Step 4.2.2**: Finalize OCI serverless image generation design (< 100 ms cold-start).
    *   **Step 4.2.3**: Specify profile-guided optimization and advanced tree-shaking strategies.
        *   *Documentation*: `BACKEND_OPTIMIZATIONS_FINAL.md`.

*   **Module 4.3: Developer Experience Enhancements Design**
    *   **Step 4.3.1**: Specify targets for IDE hot-reload (≤ 50 ms).
    *   **Step 4.3.2**: Refine inline AI error explanation mechanisms.
    *   **Step 4.3.3**: Design stable build options for desktop (native windowing / Electron-lite).
        *   *Documentation*: `DEV_EXPERIENCE_FINAL.md`.

*   **Module 4.4: Governance Foundation Launch Plan**
    *   **Step 4.4.1**: Outline legal entity structure for foundation.
    *   **Step 4.4.2**: Design open RFC process for language evolution.
    *   **Step 4.4.3**: Specify public ledger requirements for ballots/transcripts.
    *   **Step 4.4.4**: Codify governance rules: vote caps, citizen delegates.
    *   **Step 4.4.5**: Confirm default license (Apache-2.0 + LLVM exception).
    *   **Step 4.4.6**: Design PR labeling conventions (*human*, *AI-assisted*, *bot*).
        *   *Documentation*: `GOVERNANCE_MODEL.md`.

*   **Module 4.5: Teaching Stack - Month-1 & Comprehensive Documentation Plan**
    *   **Step 4.5.1**: Outline "Month-1" tutorial content (cross-platform GUI + serverless).
    *   **Step 4.5.2**: Plan comprehensive language reference, API docs, ensuring accessibility.
        *   *Documentation*: `TEACHING_MATERIALS_COMPREHENSIVE_PLAN.md`.

*   **Module 4.6: Benchmarks & Success Metrics Definition**
    *   **Step 4.6.1**: Define official SPEC benchmark suite and targets (≥ 80% C).
    *   **Step 4.6.2**: Define methodology for measuring onboarding time (< 2 hours).
    *   **Step 4.6.3**: Define metrics for tracking memory-safety bugs.
    *   **Step 4.6.4**: Define metrics for AI-assisted LOC/hour (≥ 3x Python).
        *   *Documentation*: `BENCHMARKS_SUCCESS_METRICS.md`.

*   **Module 4.7: Long-Term Support (LTS) Policy Draft**
    *   **Step 4.7.1**: Draft the LTS policy for v1.0.
        *   *Documentation*: `LTS_POLICY_V1.0.md`.

---

## Continuous Activities (Documentation & Planning)

*   **Risk Management Log**: Maintain `RISK_LOG.md` (based on Brief v4 and `Steps.md`), regularly updating it with mitigations.
    *   *Documentation*: `RISK_LOG.md`.
*   **Adoption Strategy Plan**: Maintain `ADOPTION_STRATEGY.md` detailing DevRel, challenges, grants, partnerships.
    *   *Documentation*: `ADOPTION_STRATEGY.md`.
*   **Overall Project Documentation Map**: Maintain a `PROJECT_DOCS_MAP.md` that lists all key design documents and their current status.
    *   *Documentation*: `PROJECT_DOCS_MAP.md`.

This plan will be our roadmap. We will create and populate the mentioned documentation files as we progress through each module and step. 