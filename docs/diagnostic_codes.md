# Ferra Diagnostic Codes Registry v0.1

> **Status:** Initial Draft - Related to Module 1.2 (DESIGN_DIAGNOSTICS.md)

## 1. Introduction

This document serves as the central registry for all diagnostic codes produced by the Ferra compiler. Assigning unique codes to diagnostics helps in:

*   Providing stable identifiers for errors and warnings.
*   Allowing users to look up detailed explanations for specific diagnostics.
*   Enabling tools to programmatically interpret or categorize diagnostics.
*   Facilitating easier searching and discussion of specific compiler messages.

This document is referenced as **DIAG-CODES-DOC-1** in other specifications.

## 2. Code Structure and Ranges

Diagnostic codes in Ferra will follow a general pattern, typically a letter followed by a number (e.g., `E001` for an error, `W001` for a warning).

The numeric part of the codes will be organized into ranges based on the compiler phase or category of the diagnostic. This helps in organization and allows for easier assignment of new codes.

**Proposed Initial Ranges (v0.1):**

*   **E000 - E099**: Lexical Errors (e.g., invalid character, unterminated string/comment, indentation errors)
*   **W000 - W099**: Lexical Warnings

*   **E100 - E199**: Parse Errors (Syntax Errors)
*   **W100 - W199**: Parse Warnings (e.g., ambiguous constructs if any are warned on)

*   **E200 - E299**: Semantic Errors & Name Resolution Errors (e.g., undefined variable, duplicate definition)
*   **W200 - W299**: Semantic Warnings & Name Resolution Warnings

*   **E300 - E399**: Type Errors (e.g., type mismatch, occurs check, trait bound not satisfied)
*   **W300 - W399**: Type Warnings

*   **E400 - E499**: Borrow Checking & Ownership Errors
*   **W400 - W499**: Borrow Checking & Ownership Warnings

*   **E500 - E599**: IR Generation Errors
*   **W500 - W599**: IR Generation Warnings

*   **E600 - E699**: Backend & Code Generation Errors
*   **W600 - W699**: Backend & Code Generation Warnings

*   **E700 - E799**: I/O and Filesystem Errors (from compiler operations)
*   **W700 - W799**: I/O and Filesystem Warnings

*   **E800 - E899**: Tooling Errors (e.g., `lang new` failures, config errors)
*   **W800 - W899**: Tooling Warnings

*   **L000 - L099**: Linter Messages (separate from compiler errors/warnings, can be Info, Warning, or Error severity)

## 3. Diagnostic Code List

*(This section will be populated as specific codes are defined and implemented.)*

| Code   | Severity | Phase / Category | Brief Description                                     | Associated Document Section(s) |
|--------|----------|------------------|-------------------------------------------------------|--------------------------------|
| *TBD*  | *TBD*    | *TBD*            | *TBD*                                                 | *TBD*                          |
| E001   | Error    | Lexical          | Invalid character detected in source.                 | `DESIGN_LEXER.md`              |
| E002   | Error    | Lexical          | Unterminated string literal.                          | `DESIGN_LEXER.md`              |
| E003   | Error    | Lexical          | Unterminated block comment.                           | `DESIGN_LEXER.md`              |
| E004   | Error    | Lexical          | Mixed tabs and spaces in indentation prefix.          | `DESIGN_LEXER.md`              |
| E005   | Error    | Lexical          | Dedent to an unexpected indentation level.            | `DESIGN_LEXER.md`              |
| E006   | Error    | Lexical          | Invalid escape sequence in string or char literal.    | `DESIGN_LEXER.md`              |
| E007   | Error    | Lexical          | Empty character literal.                              | `DESIGN_LEXER.md`              |
| E008   | Error    | Lexical          | Multi-character literal (not a valid single char).    | `DESIGN_LEXER.md`              |
| E009   | Error    | Lexical          | Unterminated character literal.                       | `DESIGN_LEXER.md`              |
| E400   | Error    | Ownership/Borrow | Cannot borrow data as mutable because it is already borrowed as immutable. | `OWNERSHIP_BORROW_CHECKER.md` §4.2 |
| E401   | Error    | Ownership/Borrow | Use of moved value.                                   | `OWNERSHIP_BORROW_CHECKER.md` §4.2 |
| E402   | Error    | Ownership/Borrow | Borrowed value does not live long enough (dangling reference). | `OWNERSHIP_BORROW_CHECKER.md` §4.2 |
| ...    | ...      | ...              | ...                                                   | ...                            |

## Energy Profiler & Test Budget Diagnostics (ETxxx)

| Code  | Message Template                                                                 | Notes                                                                        |
|-------|----------------------------------------------------------------------------------|------------------------------------------------------------------------------|
| ET001 | Energy budget exceeded for test '{test_name}'. Expected <= {budget_J} J, actual = {actual_J} J. | Reported by test runner when energy profiler output exceeds defined budget. |
| ET002 | Energy profiler warning: {warning_message}                                       | General warnings from the energy profiler pass (e.g., about model limitations for specific code). |
| ...   | ...                                                                              |                                                                              |

## Security & Permission Diagnostics (SExxx, SWxxx)

| Code  | Severity | Message Template                                                                                      | Notes                                                                                             |
|-------|----------|-------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------|
| SE001 | Error    | Required permission '{permission_string}' not declared in manifest for operation at {location}.         | Compile-time check failure. See `SECURITY_MODEL.md`.                                                |
| SE002 | Error    | Runtime permission denied for operation '{operation}' on resource '{resource}'. Required: '{permission_string}'. | Runtime check failure. See `SECURITY_MODEL.md`.                                                     |
| SW001 | Warning  | Declared permission '{permission_string}' appears unused by the package.                                | Potential for `lang doctor security` or build analysis. See `SECURITY_MODEL.md`.                    |
| SW002 | Warning  | Dependency '{dep_name}' requests sensitive permission '{permission_string}'. Consider auditing.           | Informational warning from `lang permissions audit`. See `SECURITY_MODEL.md`.                       |
| SE003 | Error    | Sandbox violation: Process attempted syscall {syscall_name}({syscall_args}) which is denied by profile. | Critical runtime error when seccomp-bpf (or similar) blocks an operation. See `SECURITY_MODEL.md`. |
| ...   | ...      | ...                                                                                                   |                                                                                                   |

## 4. Adding New Codes

When a new diagnostic is introduced in the compiler:
1.  Identify the appropriate category and severity.
2.  Assign the next available numeric code within that range.
3.  Add an entry to the table in Section 3 of this document, including:
    *   The code itself.
    *   Severity (Error, Warning, Note/Info).
    *   Phase/Category.
    *   A brief, stable description of what the diagnostic means.
    *   Optionally, references to sections in other design documents that detail the condition.
4.  Ensure the compiler emits this code as part of its structured diagnostic output (as per `DESIGN_DIAGNOSTICS.md`).

--- 