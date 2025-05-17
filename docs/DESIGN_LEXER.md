# Ferra Lexer Design v0.1

This document specifies the design for the Ferra v0.1 lexer (tokenizer). The lexer is the first stage of the Ferra compiler front-end, responsible for converting a stream of source characters into a stream of tokens.

This design is based on the token definitions in `docs/rfc/RFC-001_SYNTAX_GRAMMAR.md` and incorporates architectural suggestions from `docs/Steps.md` (e.g., using Ragel).

## 1. Introduction & Goals

*   **Purpose**: To scan Ferra source code and produce a sequence of tokens representing lexical units like keywords, identifiers, literals, operators, and punctuation.
*   **Accuracy**: Correctly identify all lexical elements as defined in the Ferra grammar.
*   **Performance**: Efficient tokenization is important, especially for large source files. The choice of Ragel aims to contribute to this.
*   **Error Reporting**: Provide information for reporting lexical errors (e.g., invalid characters, unterminated literals) with accurate source locations.
*   **Indentation Sensitivity**: Correctly process significant indentation and emit `INDENT` and `DEDENT` tokens.

## 2. Token Set

The lexer **MUST** recognize and produce tokens corresponding to all terminal symbols defined in `docs/rfc/RFC-001_SYNTAX_GRAMMAR.md`, Section 1.3 ("Tokens"). This includes:

*   **Keywords**: `let`, `var`, `fn`, `async`, `data`, `match`, `true`, `false`, `and`, `or`, etc.
*   **Identifiers**: `IDENTIFIER` (Unicode ID_Start/ID_Continue based).
*   **Literals**:
    *   `IntegerLiteral` (Decimal, Hex, Octal, Binary, with `_` separators).
    *   `FloatLiteral` (Decimal, with `_` separators, optional exponent).
    *   `StringLiteral` (Standard strings with escape sequences including `\u{...}`).
    *   `BooleanLiteral` (Handled by `true`/`false` keywords).
*   **Punctuation & Operators**: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `&`, `|`, `^`, `<<`, `>>`, `??` (COALESCE), `=`, `+=`, `-=`, `*=` `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, `!` (LOGICAL_NOT/MACRO_BANG), `?` (ERR_PROP_POSTFIX / future TERNARY_Q), `.` (DOT), `,`, `:`, `;`, `(`, `)`, `{`, `}`, `[`, `]`, `->` (ARROW), `=>` (FAT_ARROW), `..`, `..=`, `::` (PATH_SEP), `_`.
*   **Special Tokens**:
    *   `INDENT`: Emitted when significant indentation increases.
    *   `DEDENT`: Emitted when significant indentation decreases.
    *   `NEWLINE`: Emitted for physical newlines that are not suppressed by line continuation logic (actual termination decision is by the parser).
    *   `EOF`: End-Of-File marker.
*   **Whitespace and Comments**: Generally skipped or processed to manage indentation, but not emitted as distinct value-carrying tokens to the parser (except for `NEWLINE`, `INDENT`, `DEDENT`).

Each token should carry its type, its textual representation (lexeme), and its source location (e.g., line number, column number, and potentially byte offset).

## 3. Lexical Analysis Approach

*   **Tooling**: The lexer will be implemented using **Ragel**. Ragel is a state machine compiler that generates finite-state automata (DFAs/NFAs) in C, C++, Rust, Go, etc. This allows for efficient and robust lexer generation.
*   **Output**: The Ragel-generated code will form the core of a Rust function/module that takes source text as input and produces a stream (or vector) of tokens.
*   **State Management**: The lexer will need to manage state for significant indentation (see Section 5) and potentially for multi-line constructs if complex raw strings are added later.

## 4. Handling of Whitespace, Comments, and Newlines

*   **Whitespace**: Space (U+0020) and horizontal tab (U+0009) are generally insignificant between tokens and serve as separators. They are consumed and ignored by the lexer unless they form part of the indentation at the beginning of a line.
*   **Comments**:
    *   `LineComment` (`// ... NEWLINE`): The lexer consumes and discards line comments.
    *   `BlockComment` (`/* ... */`, supporting nesting): The lexer consumes and discards block comments.
*   **Newlines**: Physical newline characters (LF, CRLF normalized to LF) are significant.
    *   The lexer **MUST** emit a `NEWLINE` token for each logical line break that isn't part of a multi-line token (like a future multi-line string). This `NEWLINE` token is used by the parser for statement termination decisions and by the indentation logic.
    *   The lexer itself does not decide if a `NEWLINE` terminates a statement; that is parser logic based on preceding tokens (see `RFC-001_SYNTAX_GRAMMAR.md`, §1.1).

## 5. Significant Indentation Handling

This is a critical feature for Ferra, allowing Python-like block structures. The lexer plays a key role here by translating indentation changes into `INDENT` and `DEDENT` tokens.

*   **Indentation Stack**: The lexer maintains a stack of current indentation levels (e.g., a stack of column numbers, typically representing counts of spaces or equivalent tab widths).
    *   The stack is initialized with a 0 level (for the start of the file).
*   **Processing Lines**: At the beginning of each new non-empty, non-comment-only logical line:
    1.  Calculate the current line's indentation (number of leading spaces, tabs converted to spaces according to a fixed rule. ⚠️ **TBD (LEX-1)**: Define tab stop policy, e.g., 1 tab = 4 spaces, and error handling for mixed tabs/spaces for indentation. See Open Question LEX-1).
    2.  Compare this indentation with the top of the indentation stack:
        *   **Greater Indentation**: If current indent > stack top: Push current indent onto the stack and emit an `INDENT` token.
        *   **Lesser Indentation**: If current indent < stack top: Pop levels from the stack until stack top <= current indent. For each level popped where stack top was > current indent, emit a `DEDENT` token. If, after popping, stack top != current indent, it's an indentation error (a "dedent to an unexpected level").
        *   **Equal Indentation**: No `INDENT` or `DEDENT` tokens are emitted. The line simply continues at the current indentation level.
*   **End Of File (EOF)**: Before emitting the `EOF` token, if the indentation stack contains levels greater than the initial 0, emit corresponding `DEDENT` tokens to close all open indented blocks.
*   **Empty/Comment Lines**: Lines that are empty or contain only whitespace and/or comments do not affect the indentation level and do not cause `INDENT`/`DEDENT` tokens to be emitted.
*   **Hygiene**: The grammar (`RFC-001`, §1.1, "Style Hygiene") specifies that a single block must choose one style (braces or indent). The lexer produces `INDENT`/`DEDENT` tokens regardless; the parser will enforce this hygiene rule.

## 6. Identifier and Keyword Recognition

*   **Identifiers**: Recognized according to the `IDENTIFIER ::= ID_START (ID_CONTINUE)*` rule from `RFC-001_SYNTAX_GRAMMAR.md`.
    *   The lexer **MUST** correctly implement Unicode ID_Start and ID_Continue properties as per Unicode Standard Annex #31.
    *   The lexer normalises identifier lexemes to NFC (Normalization Form C) but does not perform case-folding.
    *   The lexeme (actual text) is stored with the `IDENTIFIER` token.
*   **Keywords**: After an identifier-like sequence is lexed, it is checked against a list of reserved keywords.
    *   If it matches a keyword, the corresponding keyword token is emitted (e.g., `Token::KeywordLet`).
    *   Otherwise, an `IDENTIFIER` token is emitted.
*   **Lexer Aliases**: The keywords `and` and `or` are lexed as distinct keyword tokens initially (e.g., `Token::KeywordAnd`, `Token::KeywordOr`) or the lexer can directly emit the tokens for `&&` and `||` respectively. The parser will then treat them as equivalent to `&&` and `||`. (⚠️ **TBD (LEX-2)**: Finalize if alias processing is lexer-side token rewrite or parser-side alternative handling for these keywords).

## 7. Literal Value Parsing

For literals, the lexer must recognize the syntax and also typically convert the lexeme into an internal representation or validate its format.

*   **String Literals (`StringLiteral`)**:
    *   Recognize `"..."` syntax.
    *   Process escape sequences (`\n`, `\t`, `\\`, `\"`, `\u{...}`).
    *   The token should store the *processed* string value (escapes resolved).
    *   Handle unterminated string literals as errors.
*   **Integer Literals (`IntegerLiteral`)**:
    *   Recognize decimal, hexadecimal (`0x`), octal (`0o`), and binary (`0b`) formats.
    *   Ignore underscores `_` within the number.
    *   The token should store the numeric value (e.g., as `i64` or `u64`, or a BigInt if arbitrary precision is eventually supported, but target `i64` for v0.1 `Int`).
    *   Handle invalid formats (e.g., `0xG`, `0_o7`) or out-of-range values as errors or pass to parser for semantic validation.
*   **Float Literals (`FloatLiteral`)**:
    *   Recognize decimal format, including optional exponent (`e` or `E`).
    *   Ignore underscores `_`.
    *   The token should store the numeric value (e.g., as `f64`).
    *   Handle invalid formats.
*   **Boolean Literals**: Handled as keywords `true` and `false`.

## 8. Operator and Punctuation Recognition

*   The lexer will use rules to match the various operators and punctuation symbols defined in `RFC-001_SYNTAX_GRAMMAR.md` (e.g., `+`, `->`, `&&`, `..=`).
*   Care must be taken with multi-character operators (e.g., `==` vs `=`, `->` vs `-`, `..=` vs `..`) to ensure longest match (maximal munch principle).

## 9. Error Handling

The lexer should be able to report lexical errors, including:

*   Invalid or unexpected characters.
*   Unterminated string literals.
*   Unterminated block comments.
*   Invalid numeric literal formats (if not handled by parser validation).
*   Inconsistent indentation (e.g., dedenting to an unknown level, or potentially mixed tabs/spaces if a strict policy is enforced by the lexer).

Error reports should include precise source location information. All diagnostics should strive to follow the 'positive-first' messaging style where applicable: highlight any valid prefix or context before underlining the invalid span, guiding the user towards a correct form. The lexer might attempt error recovery by skipping the invalid character/construct and trying to resume tokenization, or it might halt on the first error, depending on the desired compiler robustness.

## 10. Lexer Output (Token Stream)

The lexer will produce a stream (or list/vector) of `Token` objects. Each `Token` object should contain at least:

*   **Type**: An enum variant representing the kind of token (e.g., `KeywordLet`, `Identifier`, `IntegerLiteral`, `PlusOp`, `Indent`, `Newline`, `Eof`).
*   **Lexeme**: The actual source text corresponding to the token (e.g., `"let"`, `"my_var"`, `"123"`, `"+"`). This is crucial for error messages and potentially for some parser actions.
*   **Value (for literals)**: The processed value of literals (e.g., the integer `123`, the float `3.14`, the string `"hello\n"`).
*   **Source Location**: Start and end position (line, column, byte offset) of the token in the source file. This is essential for diagnostics.

## 11. Open Questions & Future Considerations (Lexer Specific)

*   **(LEX-1)** Define precise tab stop policy (e.g., 1 tab = 4 spaces) and error handling strategy for mixed tabs/spaces used for indentation.
*   **(LEX-2)** Finalize if `and`/`or` keyword-to-operator-token conversion happens in the lexer or is handled as alternatives by the parser.
*   Support for raw string literals (e.g., `r"..."`) or multi-line string literals (e.g., `"""..."""`).
*   Support for character literals (e.g., `'a'`).
*   Detailed parsing of float exponents and support for hexadecimal/binary float literals if added.
*   Strategy for handling shebangs (`#!...`) at the start of script files (e.g., treat as a special kind of comment on the first line).
*   Lexer performance benchmarks and optimization strategies (#LX-perf).
*   Consider providing a conceptual Ragel snippet for identifier recognition (e.g., `ID_START = [Lu Lm Lo Lt Nl]; ID_CONTINUE = [ID_START Mn Mc Nd Pc]; IDENTIFIER = ID_START ID_CONTINUE*;`) in an appendix or supplementary document for implementers. 