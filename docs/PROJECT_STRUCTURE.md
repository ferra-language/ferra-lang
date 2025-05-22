# Ferra Project Structure

## Directory Structure
```
ferra-lang/
├── Cargo.toml              # Workspace root config
├── Cargo.lock              # Lockfile (auto-generated)
├── rust-toolchain.toml     # Rust toolchain pinning
├── README.md               # Project overview
├── CONTRIBUTING.md         # Contribution guide
├── CODING_STANDARDS.md     # Coding standards
├── lang-spec-v4.yaml       # Language spec (YAML)
├── .gitignore              # Git ignore rules
├── .github/                # GitHub config, CI, and templates
│   ├── workflows/          # GitHub Actions workflows (ci.yml)
│   ├── ISSUE_TEMPLATE/     # Issue templates (bug, feature)
│   └── PULL_REQUEST_TEMPLATE.md # PR template
├── crates/                 # Rust workspace subcrates
│   └── ferra_lexer/        # Example: lexer crate
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── docs/                   # All design docs, specs, and plans
│   ├── PROJECT_STRUCTURE.md
│   ├── ... (other docs)
│   └── Other/
└── target/                 # Build artifacts (ignored in VCS)
```

- **Cargo.toml**: Workspace root, lists all crates in `crates/`.
- **crates/**: Each subcrate (e.g., `ferra_lexer`) is a Rust library or binary, with its own `Cargo.toml` and `src/`.
- **.github/**: Contains CI workflows, issue templates, and PR template.
- **docs/**: All project documentation, specs, and design docs.
- **target/**: Build output (should be gitignored).
- **rust-toolchain.toml**: Pins Rust toolchain and components (rustfmt, clippy).
- **README.md, CONTRIBUTING.md, CODING_STANDARDS.md**: Top-level project info and guidelines.

## Notes
- All new code should go in subcrates under `crates/`.
- CI is configured via `.github/workflows/ci.yml`.
- Issues and PRs use templates in `.github/ISSUE_TEMPLATE/` and `.github/PULL_REQUEST_TEMPLATE.md`.
- All documentation/specs live in `docs/`.

## Implementation Status

### Phase 1: MVP Compiler & Tooling (Q3 2025)

#### Module 1.1: Front-End - Lexer & Parser Design 🔄
- [x] Project structure setup
- [x] Initial lexer implementation
  - [x] Token definitions
  - [x] Basic lexer operations
  - [x] Test cases
- [ ] Parser implementation
  - [ ] Pratt parser for expressions
  - [ ] GLR fallback for ambiguous constructs
  - [ ] AST structure
- [ ] Integration tests

#### Module 1.2: Front-End - Type Inference & Basic Diagnostics Design
- [ ] Type inference system
  - [ ] Hindley-Milner implementation
  - [ ] Gradual typing support
- [ ] Diagnostic system
  - [ ] Error reporting
  - [ ] Warning system

#### Module 1.3: Mid-End - SSA IR Design
- [ ] SSA IR structure
- [ ] AST to IR conversion
- [ ] Semantic tags implementation

#### Module 1.4: Back-End - Initial Target Design
- [ ] LLVM backend setup
- [ ] IR to LLVM conversion
- [ ] Basic code generation

#### Module 1.5: Core Standard Library
- [ ] I/O APIs
- [ ] Basic data structures

#### Module 1.6: AI API
- [ ] `ai::ast()` implementation
- [ ] `.note.ai` section handling

#### Module 1.7: VSCode Plugin
- [ ] Syntax highlighting
- [ ] Error integration
- [ ] Project scaffolding

## Current Focus
Currently implementing Module 1.1 (Front-End - Lexer & Parser Design):
1. ✅ Basic lexer implementation with token definitions
2. 🔄 Working on parser implementation using Pratt/GLR approach
3. Next: AST structure and type inference system

## Dependencies
- LLVM 16.0
- Rust 2021 edition
- Key crates:
  - logos (lexer)
  - inkwell (LLVM bindings)
  - thiserror (error handling)
  - serde (serialization)
  - clap (CLI) 