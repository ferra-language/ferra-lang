# Ferra VSCode Plugin (Alpha) Specification v0.1

> **Status:** Initial Draft - Module 1.7 · Steps 1.7.1 & 1.7.2

## 1. Introduction

This document outlines the features and design for the alpha version of the Ferra VSCode plugin. The primary goal for the alpha is to provide essential developer support for writing Ferra code, including syntax highlighting, basic error reporting, and project creation. The plugin will actively watch `**/*.ferra` files to trigger builds/checks for diagnostic purposes.

## 2. Core Features (Step 1.7.1)

### 2.1. Basic Syntax Highlighting

*   **Goal**: Provide accurate and responsive syntax highlighting for Ferra v0.1 grammar.
*   **Mechanism**:
    *   Utilize a TextMate grammar file (`.tmLanguage.json` or similar) based on `docs/SYNTAX_GRAMMAR_V0.1.md`. The grammar file will typically reside at `syntaxes/ferra.tmLanguage.json` within the plugin structure.
    *   Define scopes for keywords, literals (strings, numbers, booleans), comments, operators, identifiers, types, function names, etc. Refer to the token list in `DESIGN_LEXER.md` for the canonical list of keywords and operators.
    *   Ensure highlighting works for both brace-based and indentation-based block styles.
*   **Considerations**:
    *   Performance for large files.
    *   Color theme compatibility (should work reasonably with popular VSCode themes).

### 2.2. Compiler Error Integration & Basic Diagnostics

*   **Goal**: Display syntax and basic type errors from the Ferra compiler directly within VSCode.
*   **Mechanism**:
    *   The plugin will invoke the Ferra compiler (e.g., `ferra check` or `ferra build`) on file save (if `ferra.diagnosticsOnSave` is true) or on demand via a command.
    *   Parse the compiler's diagnostic output. The compiler will output **one JSON object per line** for each diagnostic.
    *   Compiler diagnostics provide `lo/hi` byte offsets; the plugin will convert these to VS Code `Range` objects (using 0-indexed line and column numbers).
    *   Map error locations to VSCode's problem reporting API.
    *   Display errors as squiggles in the editor and in the "Problems" panel.
*   **Diagnostic Output Format (Compiler Requirement)**:
    *   The Ferra compiler must provide diagnostics in a machine-readable format (e.g., JSON lines). Each diagnostic should include:
        *   `severity`: (e.g., "error", "warning", "note") - See mapping table below.
        *   `message`: Human-readable error description
        *   `file_path`: Absolute path to the file
        *   `span`: `{ "lo": int, "hi": int }` (byte offsets, to be converted by plugin to line/column `Range`)
        *   `code`: Optional error code (e.g., "E001")
        *   `help`: Optional string containing further explanation or a link to documentation, suitable for hover tooltips.
*   **Severity Mapping**:

    | Compiler Severity | VS Code DiagnosticSeverity |
    |-------------------|----------------------------|
    | `error`           | `Error`                    |
    | `warning`         | `Warning`                  |
    | `note`            | `Information`              |

*   **Out of Scope for Alpha**:
    *   Real-time, as-you-type diagnostics (language server protocol features).
    *   Quick fixes or code actions.
    *   Semantic highlighting beyond TextMate grammar.

## 3. Project Scaffolding (Step 1.7.2)

### 3.1. `lang new myapp` Command Integration

*   **Goal**: Provide a VSCode command to execute the Ferra project scaffolding tool.
*   **Mechanism**:
    *   The plugin will register a VSCode command `ferra.newProject` (e.g., accessible as "Ferra: New Project" in the command palette).
    *   When invoked, this command will:
        1.  Prompt the user for a project name (e.g., `myapp`).
        2.  Prompt the user for a directory to create the project in (defaulting to the current workspace or a user-configurable path).
        3.  Execute the `lang new <project_name>` command in the chosen directory.
        4.  Optionally, open the newly created project folder in VSCode.
        5.  After successful scaffolding, ideally trigger an initial `ferra build` in the new project directory.
*   **`lang new` Tool (Compiler/CLI Requirement)**:
    *   The Ferra toolchain must provide a command `lang new <project_name>` that:
        *   Creates a new directory named `<project_name>`.
        *   Populates it with a minimal Ferra project structure (using only the self-hosting subset, see Module 1.8):
            *   `src/main.ferra` (e.g., `fn main() { println("Hello, Ferra!"); }`)
            *   `Ferra.toml` (basic project manifest, name, version)
            *   `.gitignore`
*   **User Experience**:
    *   Clear prompts and feedback during project creation.
    *   Error handling if `lang new` fails.

## 4. Configuration

*   **Plugin Settings (Alpha - Minimal)**:
    *   `ferra.compilerPath`: string (default: `"ferra"`), Path to the Ferra compiler/tool executable (if not in system PATH).
    *   Optional: Default directory for `lang new`.
    *   `ferra.diagnosticsOnSave`: boolean (default: `true`), whether to automatically run checks on file save.
    *   `ferra.autoFormatOnSave`: boolean (default: `false`), (Placeholder) whether to automatically format on file save once a formatter is available.

## 5. Open Questions / TBD

| Tag             | Issue                                                                                           | Status   |
|-----------------|-------------------------------------------------------------------------------------------------|----------|
| VSCODE-ALPHA-1  | Diagnostic range format: 0-indexed or 1-indexed for line/column after conversion from byte offsets? | Resolved (0-indexed) |
| VSCODE-ALPHA-2  | Specific TextMate scopes for all Ferra v0.1 syntactic constructs.                               | Open     |
| VSCODE-ALPHA-3  | Exact structure of `Ferra.toml` generated by `lang new`.                                        | Open     |
| VSCODE-ALPHA-4  | Should the plugin bundle a specific version of the Ferra toolchain, or rely on user-installed? | Open     |
| VSCODE-ALPHA-5  | Should `ai::ast()` export be hookable from the plugin (Phase 2)?                                  | Open     |

---

This specification will guide the initial development of the Ferra VSCode plugin. 