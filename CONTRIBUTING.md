# Contributing to Ferra

> **Default branch:** This repo uses **`main`**.  
> When you fork or clone, Git will check out `main`—there is no `master` branch.

Thank you for your interest in contributing to Ferra! This guide will walk you through the process of setting up your environment, following our conventions, and submitting your changes.

---

## 1. Setup

1. Fork the upstream repo on GitHub.
2. Clone your fork (replace `<YOUR_GITHUB_USERNAME>` with the name you see in your browser's address bar after forking):
   ```bash
   git clone git@github.com:<YOUR_GITHUB_USERNAME>/ferra-lang.git
   cd ferra-lang
   ```
3. Install Rust Toolchain  
   We lock to the stable toolchain via `rust-toolchain.toml`.
   ```bash
   rustup update
   rustup default stable
   ```

---

## 2. Branching & Commits

1. **Create a feature branch** for each change, using a prefix that matches your commit type:
   - `feat/lexer-plus-token`
   - `fix/parser-empty-input`
   - `docs/contributing-guide`
   ```bash
   git checkout -b feat/lexer-plus-token
   ```
2. **Commit messages** should follow [Conventional Commits](https://www.conventionalcommits.org/), for example:
   ```
   feat(lexer): emit Plus token
   fix(parser): handle empty input
   docs: update CONTRIBUTING.md
   ```

---

## 3. Coding Standards

1. All code must comply with our [Coding Standards](./CODING_STANDARDS.md).
2. Format your code:
   ```bash
   cargo fmt -- --check
   ```
3. Lint your code:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

---

## 4. Running Tests

1. Run the full test suite and ensure everything passes:
   ```bash
   cargo test --workspace
   ```

---

## 5. Proposing Changes

1. Push your feature branch to your fork:
   ```bash
   git push origin feat/your-branch
   ```
2. Open a Pull Request against the `main` branch of the upstream repository.
3. Ensure CI checks pass (formatting, clippy, tests).
4. Describe your change clearly in the PR description and link to any related issues.

---

## 6. Issue & PR Templates

We provide templates to keep reports and PRs consistent. GitHub will automatically load the right one when you open an issue or pull request:

- 🐛 **Bug reports**: use `.github/ISSUE_TEMPLATE/bug_report.md`
- ✨ **Feature requests**: use `.github/ISSUE_TEMPLATE/feature_request.md`
- 📦 **Pull requests**: use `.github/PULL_REQUEST_TEMPLATE.md`

Refer to these whenever you file something so that all the necessary fields are filled out.

---

## 7. Design Docs & Resources

1. When working on a specific component, refer to its design doc:
   1. Lexer: [`docs/DESIGN_LEXER.md`](./docs/DESIGN_LEXER.md)
   2. Parser: [`docs/DESIGN_PARSER.md`](./docs/DESIGN_PARSER.md)
   3. AST: [`docs/AST_SPECIFICATION.md`](./docs/AST_SPECIFICATION.md)
   4. IR: [`docs/IR_SPECIFICATION.md`](./docs/IR_SPECIFICATION.md)
   5. Backend: [`docs/BACKEND_LLVM_X86-64.md`](./docs/BACKEND_LLVM_X86-64.md)
   6. Type Inference: [`docs/DESIGN_TYPE_INFERENCE.md`](./docs/DESIGN_TYPE_INFERENCE.md)
   7. Diagnostics: [`docs/DESIGN_DIAGNOSTICS.md`](./docs/DESIGN_DIAGNOSTICS.md)
   8. Ownership/Borrowing: [`docs/OWNERSHIP_BORROW_CHECKER.md`](./docs/OWNERSHIP_BORROW_CHECKER.md)
   9. Concurrency: [`docs/CONCURRENCY_MODEL.md`](./docs/CONCURRENCY_MODEL.md)
   10. ...and others in the `docs/` directory.
2. CI: [`ci.yml`](.github/workflows/ci.yml) enforces formatting, linting, and tests on every PR.

---

Thank you for helping make Ferra better! 🎉