# Ferra Lexer Crate: Implementation & Test Plan

This document tracks all code and tests to be written for the `ferra_lexer` crate, based on `docs/DESIGN_LEXER.md`.

---

## 1. Token Coverage
- [x] Some keywords (`let`, `var`, `fn`, `async`, `data`, `match`, `true`, `false`)
- [x] Identifiers (ASCII only, not Unicode yet)
- [x] Integer literals (decimal only)
- [x] Float literals (with exponents, underscores)
- [ ] String literals (escapes, Unicode)
- [ ] Character literals (escapes, Unicode)
- [ ] Boolean literals (`true`, `false`)
- [ ] Byte literals (`b'a'`, `b"foo"`)
- [x] Some single-char operators & punctuation (`=`, `;`, `+`, `-`, `*`, `/`, `,`, `:`, `(`, `)`, `{`, `}`)
- [x] All operators & punctuation (multi-char, rest of single-char)
- [ ] Comments (`// ...`, `/* ... */` with nesting)
- [ ] Indentation tokens (`Indent`, `Dedent`, `Newline`)
- [ ] Error token for unrecognized input
- [ ] **Ragel integration:** Define `.rl` state-machine spec and integrate generated Rust code (future)
- [ ] **Shebang handling:** Treat a `#!...` shebang at the very start of a file as a single-line comment (skipped)

## 2. Lexing Logic
- [x] Main lexing loop (cursor/iterator)
- [x] Whitespace skipping
- [x] Identifier/keyword recognition
- [x] Number literal recognition (int, decimal only)
- [ ] Number literal recognition (float, hex, octal, binary)
- [ ] String/char/byte literal recognition
- [ ] Operator and punctuation recognition (maximal munch, multi-char)
- [ ] Indentation stack logic for `Indent`/`Dedent`
- [ ] Newline handling
- [ ] Error handling and recovery
- [x] Source location tracking (basic)
  - TODO: When implementing advanced span/diagnostic support, reintroduce byte offset tracking (variable: `offset`) and character consumption tracking (variable: `chars_consumed`) in the lexer code for precise spans.
- [ ] **Numeric underscores:** Strip and ignore `_` in integer literals (all bases)
- [ ] **Lexer aliases:** Rewrite `and` → `LogicalAnd`, `or` → `LogicalOr` after keyword recognition
- [ ] **Skip NEWLINE/indent for blank/comment-only lines**

## 3. Unicode & Normalization
- [ ] Unicode ID_Start/ID_Continue for identifiers
- [ ] NFC normalization for identifier lexemes

## 4. Error Handling
- [ ] Invalid character reporting
- [ ] Unterminated string/char/block comment
- [ ] Invalid numeric formats
- [ ] Indentation errors (mixed tabs/spaces, dedent to unknown level)
- [ ] Positive-first error messaging
- [ ] **Nested block comments:** Correctly handle and discard nested block comments, report unterminated

## 5. Testing Strategy
- [x] Unit tests in `src/lib.rs` for basic cases
- [ ] Unit tests in `src/lib.rs` for each token type and edge case
- [ ] Integration tests in `tests/` directory:
    - [ ] Full-file lexing scenarios
    - [ ] Error cases and recovery
    - [ ] Indentation/whitespace scenarios
    - [ ] Unicode identifier tests
    - [ ] Comment and doc extraction
- [ ] Fuzz tests (optional, for robustness)
- [ ] **Test multi-line tokens:** Ensure correct start/end positions for multi-line tokens (block comments, raw strings, etc.)

## 6. Performance & Robustness (Future)
- [ ] Efficient handling of large files
- [ ] Lexer benchmarks
- [ ] Error recovery strategies
- [ ] (Future) Hexadecimal/binary float literal syntax

---

**Update this checklist as you implement and test each feature.**

Integration test: `tests/multi_char_ops.rs`

Integration test: `tests/float_literals.rs`