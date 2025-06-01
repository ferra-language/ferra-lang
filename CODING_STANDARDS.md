# Ferra Language Parser - Coding Standards & Implementation Status

## Project Status: ✅ PHASE 2.8 COMPLETE - All Medium-Term Features Implemented

### Recent Achievements (Latest Update)
- **Complex Nested Attributes**: ✅ Full support for complex attribute expressions and field attributes
- **Error Recovery Stress Testing**: ✅ Comprehensive error boundary testing with 6 test scenarios
- **Memory Profiling Infrastructure**: ✅ Complete memory tracking with 8 profiling test cases
- **Enhanced Return Statement Parsing**: ✅ Support for complex expressions in return statements
- **Code Quality**: ✅ All clippy warnings resolved, clean formatting

### Test Suite Status: 500+ Tests Passing
- **Unit Tests**: 63 core library tests
- **Integration Tests**: 24 comprehensive test modules covering all parser features
- **Medium-Term Features**: 25 new tests (11 attribute + 6 stress + 8 memory)
- **Legacy Compatibility**: All existing functionality preserved

---

## Implementation Overview

This document outlines the initial coding standards for the development of the Ferra compiler and its associated tooling, which will primarily be written in Rust.

## Medium-Term Features Implementation Details

### 1. Complex Nested Attributes (`test_complex_nested_attributes.rs`)

**Status**: ✅ COMPLETE - 11/11 tests passing

**Features Implemented**:
- **Enhanced Attribute Parser**: Support for complex expressions in attribute arguments (e.g., `feature = "test"`, nested parentheses)
- **Field-Level Attributes**: Attributes can now be applied to data class fields with proper parsing
- **Complex Expression Support**: Full integration with the Pratt expression parser for attribute values
- **Whitespace Tolerance**: Flexible parsing of attributes with various formatting styles

**Key Technical Improvements**:
- Updated `parse_attribute_argument()` to handle complex token sequences
- Added attribute support to field parsing in `parse_field()`
- Enhanced `parse_return_statement()` to use full expression parser instead of token-by-token approach

### 2. Error Recovery Stress Testing (`test_error_recovery_stress.rs`)

**Status**: ✅ COMPLETE - 6/6 tests passing

**Test Scenarios Implemented**:
- **Massive Syntax Error Cascade**: 100 functions with 10% error rate
- **Deeply Nested Error Recovery**: 25 levels of nesting with scattered errors  
- **Malformed Expression Storm**: 100 malformed expressions with various error types
- **Error Recovery Boundary Conditions**: Edge cases (empty files, only delimiters)
- **High Frequency Error Bursts**: Alternating valid/invalid code sections
- **Error Recovery Scalability**: Performance scaling from 25 to 250 errors

**Architecture**:
- Comprehensive error token generation for different scenarios
- Integration with existing error recovery infrastructure
- Performance monitoring for large-scale error scenarios

### 3. Memory Profiling Infrastructure (`test_memory_profiling.rs`)

**Status**: ✅ COMPLETE - 8/8 tests passing

**Profiling Capabilities**:
- **Memory Tracking**: `MemoryTracker` struct for monitoring parser memory usage
- **Scalability Testing**: Programs from 50 to 2000 functions
- **Memory Leak Detection**: Multi-iteration parsing with memory monitoring
- **Arena Efficiency Testing**: Allocation patterns and reuse optimization
- **Pathological Case Testing**: Deep nesting and wide structure scenarios

**Test Coverage**:
- Baseline memory usage for small programs (50 functions)
- Memory scaling validation (linear vs exponential growth)
- Arena allocation efficiency comparison
- Very large program memory limits (1000+ functions)

---

These standards aim to ensure code quality, consistency, readability, and maintainability across the project.

## 1. Formatting

*   **`rustfmt`**: All Rust code **MUST** be formatted using the latest stable version of `rustfmt` with its default settings.
    *   If any deviations from the default `rustfmt` settings (e.g., `max_width`) are strictly necessary and agreed upon, they **MUST** be configured in a `rustfmt.toml` file at the project root. Otherwise, no `rustfmt.toml` is needed.
    *   It is recommended to configure your IDE to format on save using `rustfmt`.
    *   CI checks will enforce `rustfmt` compliance.

## 2. Linting

*   **`clippy`**: All Rust code **MUST** pass linting checks from the latest stable version of `clippy` using its default set of lints (`clippy::all`).
    *   Strive to address all `clippy` warnings. If a specific lint is deemed inappropriate for a particular piece of code, it can be locally disabled with `#[allow(clippy::lint_name)]` accompanied by a comment explaining the rationale.
    *   CI checks will enforce `clippy` compliance.

## 3. Naming Conventions

*   Follow the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) for naming conventions, especially for public APIs.
    *   **Modules, crates, types (structs, enums, traits), and type aliases**: `UpperCamelCase` (e.g., `Lexer`, `SyntaxNode`, `ParseError`).
    *   **Functions, methods, variables, and statics**: `snake_case` (e.g., `parse_token`, `current_char`, `MAX_ERRORS`).
    *   **Macros**: `snake_case!` for function-like macros, `UpperCamelCase!` for derive macros.
    *   **Constants**: `UPPER_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT`).
    *   **Generic type parameters**: Concise `UpperCamelCase`, typically single letters like `T`, `E`, `K`, `V`.

## 4. Comments

*   **Purpose**: Write comments to explain *why* code is written a certain way, not *what* it does (the code itself should clearly express the *what*).
*   **Doc Comments**: Use Markdown for documentation comments (`///` for items, `//!` for modules/crates).
    *   All public functions, structs, enums, traits, and modules **SHOULD** have documentation comments explaining their purpose, parameters (if any), return values (if any), and any panics or important side effects.
    *   Include usage examples in doc comments where helpful (````rust ... ````).
*   **Regular Comments**: Use `//` for line comments and `/* ... */` for block comments sparingly. Prefer explaining complex logic through well-named variables and functions.

## 5. Error Handling

*   **`Result<T, E>`**: Prefer using `Result<T, E>` for functions that can fail.
*   **Custom Error Types**: Define custom error types (enums or structs implementing `std::error::Error`) for different modules or error categories. This provides more context than generic error strings.
    *   Ensure error types are descriptive and, where possible, implement `Display` and `Debug`.
*   **`panic!`**: Avoid `panic!` in library code that can be recovered from. `panic!` is generally reserved for unrecoverable errors (e.g., logic errors indicating a bug in the program itself, broken invariants).
*   **Positive-First Messaging**: Align with Ferra's language design goal of "positive-first error messaging." While this applies more to Ferra's compiler diagnostics for its users, the principle of clear, actionable, and non-blaming error messages is also good practice for the compiler's own internal error handling and logging.

## 6. Modularity & Crates

*   Organize code into logical modules and crates.
*   Clearly define public APIs for each module/crate and minimize unnecessary public exposure.
*   Consider creating internal crates within a workspace for larger distinct components of the compiler (e.g., `ferra_lexer`, `ferra_parser`, `ferra_type_checker`).

## 7. Testing

*   Write unit tests for individual functions and modules.
*   Write integration tests to check interactions between components.
*   Tests should be placed in a `tests` submodule (e.g., `#[cfg(test)] mod tests { ... }`) or in an integration tests directory (`tests/`).
*   Strive for high test coverage.

## 8. Dependencies

*   Minimize external dependencies.
*   Carefully vet any new dependency for its maintenance status, security, and performance implications.
*   Keep dependencies up-to-date, but test thoroughly before upgrading.

## 9. Unsafe Code

*   Avoid `unsafe` Rust unless absolutely necessary and its use can be rigorously justified (e.g., FFI, specific low-level optimizations after profiling).
*   Any use of `unsafe` **MUST** be clearly documented with comments explaining why it's needed and what invariants must be upheld to maintain safety.

## 10. Commit Messages

*   Follow the Conventional Commits specification (e.g., `feat: ...`, `fix: ...`, `docs: ...`, `style: ...`, `refactor: ...`, `test: ...`, `chore: ...`).
*   The commit message subject line should be concise (<= 50 chars).
*   Provide more detail in the commit message body if necessary.

## 11. Code Reviews

*   All code changes should be reviewed by at least one other contributor before merging (once the project has multiple contributors).
*   Reviewers should check for adherence to these coding standards, correctness, performance, and maintainability.

## 12. Evolution of Standards

These standards are a starting point and may evolve as the project grows. Changes to these standards should be discussed and agreed upon by the project contributors.

## 13. Rust Toolchain

To ensure consistency across all development environments, this project uses a specific Rust toolchain version.

*   A `rust-toolchain.toml` file **MUST** be present in the project root, specifying the channel (e.g., `stable`, `nightly`). Initially, we will use `stable`.
*   All contributors **MUST** use the toolchain defined in this file. This is typically handled automatically by `rustup` if the file is present.

## 14. IDE Configuration (VS Code / rust-analyzer)

To maintain consistency and leverage Rust tooling effectively within VS Code:

*   A recommended `.vscode/settings.json` file will be provided in the repository.
*   Key settings include:
    *   `editor.formatOnSave = true` (for `rustfmt`).
    *   `rust-analyzer.checkOnSave.command = "clippy"` (or `rust-analyzer.cargo.checkOnSave.allTargets = true` depending on the exact `rust-analyzer` settings desired for clippy checks on save).
*   Contributors are encouraged to use these settings or ensure their IDE provides equivalent functionality.

## 15. Continuous Integration (CI)

A CI pipeline (e.g., using GitHub Actions) **MUST** be configured to:

*   Check formatting: `cargo fmt -- --check`
*   Run linter: `cargo clippy --all-targets -- -D warnings` (treat all warnings as errors)
*   Run tests: `cargo test --all-targets`
*   (Later) Run security audit: `cargo audit`
*   Pull requests **MUST** pass all CI checks before being considered for merging.

## 16. Development Workflow

To maintain a clean and reviewable commit history:

*   **Feature Branches**: All new work (features, bug fixes, documentation) **MUST** be done on separate feature branches. Branch names should be descriptive, e.g., `feat/parser-error-codes`, `fix/off-by-one-lexer`, `docs/readme-update`.
*   **Pull Requests (PRs)**: Changes **MUST** be submitted via Pull Requests to the `main` branch (or designated integration branch).
*   **Reviews**: At least one approving review from another contributor is required before merging (once the team grows beyond one).
*   **Squash and Merge**: PRs **SHOULD** be squashed and merged to maintain a linear and clean history on the `main` branch. The commit message for the squashed commit **MUST** follow the Conventional Commits specification (see Section 10).

## 17. Performance & Allocation Guidelines (Rust)

While premature optimization is discouraged, be mindful of performance, especially in core compiler components:

*   Avoid unnecessary `clone()` operations, particularly in hot loops or performance-sensitive code paths. Understand data ownership and borrowing to minimize cloning.
*   Prefer using iterators and their adaptors over manual indexing where idiomatic and clear.
*   Use `#[inline]` judiciously and only after profiling indicates a significant benefit for small, frequently called functions. Let the compiler make most inlining decisions.
*   Be mindful of allocations. For performance-critical sections, consider using data structures that minimize allocations or allow for arena allocation if appropriate.

## 18. Security & Supply-Chain Scanning (Rust)

Maintaining a secure supply chain is critical:

*   **Dependency Auditing**: All Rust dependencies **MUST** be regularly checked for known vulnerabilities using `cargo audit` or an equivalent tool integrated into the CI pipeline.
*   **Vulnerability Policy**: High-severity and critical advisories found in dependencies **MUST** block a PR from merging. Such vulnerabilities must be addressed by updating the dependency, replacing it, or (in rare, well-justified cases) formally accepting the risk with clear documentation.
*   **SBOM Generation**: In line with Ferra's broader goals, processes for generating a Software Bill of Materials (SBOM) for the compiler itself will be established as the project matures. 