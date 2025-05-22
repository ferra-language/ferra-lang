# Ferra

[Contributing](CONTRIBUTING.md)

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

## Getting Started

> **Note:** This repository uses **`main`** as its default branch.

1. Fork this repo on GitHub.
2. Clone your fork (replace `<YOUR_GITHUB_USERNAME>` with the name you see in your browser's address bar after forking):
   ```bash
   git clone git@github.com:<YOUR_GITHUB_USERNAME>/ferra-lang.git
   cd ferra-lang
   ```

> 🔧 _Tip:_  
> After you run the above, `git remote -v` should show your fork's URL.

## Teaching & Tutorials

- [Month-1 Fullstack Tutorial](docs/teaching/month-1/README.md)
- [Teaching Materials](docs/TEACHING_MATERIALS_INITIAL.md)

## Contributing

- See [CONTRIBUTING.md](CONTRIBUTING.md) and [GitHub Issues](https://github.com/ferra-language/ferra-lang/issues)
- Contributions, bug reports, and feedback are welcome!

---

For more, browse the `docs/` directory and the [comprehensive plan](docs/comprehensive_plan.md). 