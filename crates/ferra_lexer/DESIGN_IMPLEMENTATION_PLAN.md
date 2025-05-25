# Ferra Lexer Crate: Implementation & Test Plan

This document tracks all code and tests to be written for the `ferra_lexer` crate, based on `docs/DESIGN_LEXER.md`.

---

## 1. Token Coverage
- [x] Some keywords (`let`, `var`, `fn`, `async`, `data`, `match`, `true`, `false`)
- [x] Identifiers (ASCII and Unicode, NFC normalized)
- [x] Integer literals (decimal, hex, octal, binary, underscores)
- [x] Float literals (with exponents, underscores, trailing dot, leading dot)
- [x] String literals (basic escapes: \n, \t, \\, ", no Unicode \u{...} yet)
- [x] Character literals (basic escapes: '\', \\, \n, \r, \t, \0, no Unicode \u{...} yet)
- [x] Boolean literals (`true`, `false`) (lexed as keywords with LiteralValue::Boolean)
- [x] Byte literals (`b'a'`, `b"foo"`)
- [x] Some single-char operators & punctuation (`=`, `;`, `+`, `-`, `*`, `/`, `,`, `:`, `(`, `)`, `{`, `}`)
- [x] All operators & punctuation (multi-char, rest of single-char)
- [x] Comments (`// ...`, `/* ... */` with nesting)
- [x] Indentation tokens (`Indent`, `Dedent`, `Newline`) (Python-style)
- [x] Error token for unrecognized input (robust)
- [ ] **Ragel integration:** Define `.rl` state-machine spec and integrate generated Rust code (future)
- [x] **Shebang handling:** Treat a `#!...` shebang at the very start of a file as a single-line comment (skipped)

## 2. Lexing Logic
- [x] Main lexing loop (cursor/iterator)
- [x] Whitespace skipping
- [x] Identifier/keyword recognition (Unicode-aware, NFC normalized)
- [x] Number literal recognition (int, float, hex, octal, binary, underscores)
- [x] String literal recognition (basic escapes)
- [x] Character literal recognition (basic escapes, error handling for empty/multi/unterminated/invalid-escape)
- [x] Byte literal recognition
- [x] Operator and punctuation recognition (maximal munch, multi-char)
- [x] Indentation stack logic for `Indent`/`Dedent`
- [x] Newline handling
- [x] Error handling and recovery
- [x] Source location tracking (basic)
  - TODO: When implementing advanced span/diagnostic support, reintroduce byte offset tracking (variable: `offset`) and character consumption tracking (variable: `chars_consumed`) in the lexer code for precise spans.
- [x] **Numeric underscores:** Strip and ignore `_` in integer literals (all bases)
- [x] **Lexer aliases:** Rewrite `and` → `LogicalAnd`, `or` → `LogicalOr` after keyword recognition
- [x] **Skip NEWLINE/indent for blank/comment-only lines**
- [x] **Shebang handling:** Skip shebang line if present at file start

## 3. Unicode & Normalization
- [x] Unicode ID_Start/ID_Continue for identifiers (via unicode-ident)
- [x] NFC normalization for identifier lexemes (via unicode-normalization)

## 4. Error Handling
- [x] Invalid character reporting
- [x] Unterminated string literal
- [x] Unterminated character literal
- [x] Invalid escape sequence in char literal
- [x] Empty/Multi-character char literal errors
- [x] Unterminated block comment 
- [x] Invalid numeric formats (basic prefix/suffix errors implemented)
- [x] Indentation errors (mixed tabs/spaces, dedent to unknown level)
- [x] Positive-first error messaging (all error messages are now user-friendly and specific)
- [x] **Nested block comments:** Correctly handle and discard nested block comments, report unterminated (verified and tested)

**All error handling is production-ready, with specific, user-friendly messages and robust error token logic.**

## 5. Testing Strategy
- [x] Unit tests in `src/lib.rs` for basic cases
- [x] Unit tests in `src/lib.rs` for each token type and edge case
- [x] Integration tests in `tests/` directory:
    - [x] Full-file lexing scenarios (`tests/general_lexing.rs`, `tests/unit_basic.rs`)
    - [x] Error cases and recovery (`tests/char_literals.rs`, `tests/string_literals.rs`, `tests/comments.rs`, `tests/numeric_literals.rs`)
    - [x] Indentation/whitespace scenarios (`tests/unit_basic.rs`)
    - [x] Unicode identifier tests (`tests/keywords_identifiers.rs`)
    - [x] Comment and doc extraction (`tests/comments.rs`)
    - [x] Operator and punctuation tests (`tests/operators_punctuation.rs`, `tests/multi_char_ops.rs`)
    - [x] Float literal tests (`tests/float_literals.rs`)
    - [x] Shebang handling (`tests/general_lexing.rs`)
- [x] Fuzz tests (property-based, for robustness) (`tests/fuzz.rs`)
- [x] **Test multi-line tokens:** Ensure correct start/end positions for multi-line tokens (block comments, raw strings, etc.)

**Test files:**
- `tests/char_literals.rs`
- `tests/string_literals.rs`
- `tests/comments.rs`
- `tests/numeric_literals.rs`
- `tests/keywords_identifiers.rs`
- `tests/operators_punctuation.rs`
- `tests/multi_char_ops.rs`
- `tests/float_literals.rs`
- `tests/unit_basic.rs`
- `tests/general_lexing.rs`
- `tests/fuzz.rs`

**All tests are comprehensive, covering all error and edge cases, and a property-based fuzz test ensures the lexer never panics.**

## 6. Performance & Robustness (Future)
- [ ] Efficient handling of large files
- [ ] Lexer benchmarks
- [ ] Error recovery strategies
- [ ] (Future) Hexadecimal/binary float literal syntax

---

**All Section 4 (Error Handling) and Section 5 (Testing Strategy) items are complete and production-ready.**

Integration test: `tests/multi_char_ops.rs`
Integration test: `tests/float_literals.rs`