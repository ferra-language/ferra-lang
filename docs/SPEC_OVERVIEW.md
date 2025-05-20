# Specification Overview (lang-spec-v4.yaml)

> **Status:** Initial Draft - Module 0.3 · Step 0.3.1

## 1. Purpose

This document provides an overview and explanation for the `lang-spec-v4.yaml` file located in the project root. The `lang-spec-v4.yaml` file serves as a high-level, machine-readable summary of the Ferra language's key features, design choices, and target capabilities as envisioned in "Design Brief v4" and refined by Dr. Hoare's feedback in `Steps.md`.

It is intended to provide a quick, canonical reference for:
- Core architectural decisions (e.g., lexer/parser strategy, IR form).
- Targeted backend platforms.
- Key language features and their intended scope (e.g., concurrency model, AI API endpoints).
- Goals for developer experience, teaching, and governance.

## 2. Structure

The `lang-spec-v4.yaml` file is organized into several top-level keys, each representing a major aspect of the Ferra project:

- `date`: The last modification date of the specification summary.
- `front_end`: Details about the compiler's front-end components.
  - `lexer`: Chosen lexer generation strategy.
  - `parser`: Chosen parser generation strategy.
  - `type_inference`: High-level approach to type inference.
  - `diagnostics_latency_ms`: Target latency for diagnostics.
- `mid_ir`: Information about the mid-end Intermediate Representation.
  - `form`: The structure of the IR (e.g., SSA).
  - `tags`: Mechanism for semantic tagging.
- `back_ends`: A list of targeted backend platforms and output formats.
- `supply_chain`: Plans for software supply chain security.
  - `sbom`: Standard for Software Bill of Materials.
  - `signing`: Method for artifact signing.
- `ai_api`: Specifications for the AI-native API.
  - `endpoints`: List of planned AI functionalities.
  - `coverage_gate`: Target code coverage for AI-related features.
- `stdlib`: Overview of the standard library.
  - `domains`: Key areas the standard library will cover.
  - `pkg_mgr`: Approach for package management.
- `ffi`: List of languages targeted for Foreign Function Interface compatibility.
- `concurrency`: The chosen concurrency model.
  - `model`: e.g., deterministic actors.
  - `cpu_gpu_partition`: How CPU/GPU work might be partitioned.
- `security`: Key security features and targets.
  - `capability_permissions`: Whether capability-based security is planned.
  - `energy_budget_j`: Target energy consumption for certain operations.
- `developer_exp`: Goals for developer experience.
  - `hot_reload_ms`: Target for hot-reloading.
  - `cli_scaffold`: Availability of project scaffolding tools.
- `teaching`: Tiers of teaching materials to be developed.
- `governance`: Principles for project governance.
  - `vote_cap_percent`: Voting cap details.
  - `license`: Chosen project license.
- `benchmarks`: Key performance and adoption benchmarks.
  - `spec_ratio`: Target performance relative to benchmarks like SPEC.
  - `onboarding_hours`: Target time for new developers to become productive.

## 3. Usage

This YAML file is primarily for:
- **Quick Reference**: Provides an at-a-glance summary of project goals and technical decisions.
- **Automated Tooling (Future)**: Could be used by tools to understand the project's scope or to configure CI/CD pipelines.
- **Tracking High-Level Changes**: Updates to this file reflect shifts in major architectural decisions or project targets.

It is **not** intended to replace the detailed design documents found in the `/docs` directory. For in-depth specifications of any particular component (e.g., syntax, IR, backend), please refer to the relevant `.md` file.

## 4. Evolution

The `lang-spec-v4.yaml` file will evolve alongside the project. As design decisions are refined or new targets are adopted, this file will be updated accordingly. The `date` field helps track the version of this summary. 