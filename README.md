[![CI](https://github.com/ferra-language/ferra-lang/actions/workflows/ci.yml/badge.svg)](https://github.com/ferra-language/ferra-lang/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-latest-brightgreen.svg)](docs/)

**Quick Links:** [Contributing](CONTRIBUTING.md) · [Issues](https://github.com/ferra-language/ferra-lang/issues) · [Docs](docs/)

# Ferra

Ferra is an AI-native, general-purpose programming language designed to be as easy as Python, with Rust-class performance and memory safety. It features minimal syntax, gradual static typing, an ownership/borrow model, deterministic async/actor concurrency, and positive-first error messaging.

## Repository

- **GitHub:** [https://github.com/ferra-language/ferra-lang.git](https://github.com/ferra-language/ferra-lang.git)
- **Version Control:** Git
- **Issue Tracker:** [GitHub Issues](https://github.com/ferra-language/ferra-lang/issues)

## Project Vision

Ferra aims to make high-performance, safe, and modern programming accessible to everyone. See [docs/PROJECT_OVERVIEW.md](docs/PROJECT_OVERVIEW.md) for details.

## Project Structure

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
│   ├── ferra_lexer/        # Example: lexer crate
│   ├── ferra_parser/       # (future) parser crate
│   ├── ferra_ast/          # (future) AST crate
│   └── ... other subcrates to follow
├── docs/                   # All design docs, specs, and plans
│   ├── PROJECT_STRUCTURE.md
│   ├── ... (other docs)
│   └── Other/
```

- All new code should go in subcrates under `crates/`.
- CI is configured via `.github/workflows/ci.yml`.
- Issues and PRs use templates in `.github/ISSUE_TEMPLATE/` and `.github/PULL_REQUEST_TEMPLATE.md`.
- All documentation/specs live in `docs/`.

## Documentation

- [Project Overview](docs/PROJECT_OVERVIEW.md)
- [Project Structure](docs/PROJECT_STRUCTURE.md)
- [Comprehensive Plan](docs/comprehensive_plan.md)
- [Docs Map](docs/PROJECT_DOCS_MAP.md)
- [Language Spec](lang-spec-v4.yaml)
- [Coding Standards](CODING_STANDARDS.md)

See [docs/PROJECT_DOCS_MAP.md](docs/PROJECT_DOCS_MAP.md) for a full list of specs, RFCs, and teaching materials.

## Prerequisites
- [Rust toolchain](https://www.rust-lang.org/tools/install) (stable)
- Git

> **You must have these installed before cloning or building Ferra.**

## Getting Started

> **Note:** This repository uses **`main`** as its default branch.

1. Fork this repo on GitHub.
2. Clone your fork (replace `<YOUR_GITHUB_USERNAME>` with your username):
   - **SSH:**
     ```bash
     git clone git@github.com:<YOUR_GITHUB_USERNAME>/ferra-lang.git
     cd ferra-lang
     ```
   - **HTTPS:**
     ```bash
     git clone https://github.com/<YOUR_GITHUB_USERNAME>/ferra-lang.git
     cd ferra-lang
     ```
3. (Optional but recommended) [Sync with upstream](CONTRIBUTING.md#2-syncing-with-upstream) to keep your fork up to date.
4. Install Rust toolchain and check your setup:
   ```bash
   rustup update
   cargo fmt -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --workspace
   ```

> 🔧 _Tip:_  
> After you run the above, `git remote -v` should show your fork's URL.

## Quick Example

Try a "Hello, Ferra!" in four steps:

```bash
cargo build --workspace
# then
 echo 'print("Hello, Ferra!")' > hello.ferra
target/debug/ferrac hello.ferra
./hello
```

## Teaching & Tutorials

- [Month-1 Fullstack Tutorial](docs/teaching/month-1/README.md)
- [Teaching Materials](docs/TEACHING_MATERIALS_INITIAL.md)

## Contributing

- See [CONTRIBUTING.md](CONTRIBUTING.md) and [GitHub Issues](https://github.com/ferra-language/ferra-lang/issues)
- Contributions, bug reports, and feedback are welcome!

## License & Conduct

[![Code of Conduct](https://img.shields.io/badge/code%20of%20conduct-enforced-brightgreen.svg)](CODE_OF_CONDUCT.md)

Licensed under [Apache-2.0](https://opensource.org/licenses/Apache-2.0) — see [LICENSE](LICENSE) for details.

See our [Code of Conduct](CODE_OF_CONDUCT.md)—it's enforced on all issues and pull requests.

---

For more, browse the `docs/` directory and the [comprehensive plan](docs/comprehensive_plan.md). 