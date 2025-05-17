# Ferra Parser Design v0.1

This document specifies the design for the Ferra v0.1 parser. The parser is the second stage of the Ferra compiler front-end, following the lexer. It takes a stream of tokens (from `DESIGN_LEXER.md`) as input and produces an Abstract Syntax Tree (AST) representing the syntactic structure of the source code, based on `RFC-001_SYNTAX_GRAMMAR.md`.

## 1. Introduction & Goals

*   **Purpose**: To analyze the token stream for syntactic correctness according to the Ferra grammar and to construct an AST that accurately reflects the code's structure.
*   **Input**: A stream of tokens from the Ferra Lexer.
*   **Output**: An Abstract Syntax Tree (AST). If parsing fails, a list of syntax errors with source location information.
*   **Grammar Adherence**: Strictly follow `RFC-001_SYNTAX_GRAMMAR.md`.
*   **Error Reporting**: Provide clear, "positive-first" syntax error messages, ideally with suggestions for fixes.
*   **Extensibility**: Design with future language features in mind, though the initial focus is on v0.1 syntax.
*   **Versioning**: The parser version will align with the Ferra language specification version (e.g., parser v0.1 implements Ferra spec v0.1). Breaking changes in the language spec may necessitate breaking changes in the parser.

## 2. Overall Architecture

*   **Primary Strategy**: A **Recursive Descent** parser will be used for most top-level constructs, declarations, and statements. This approach is straightforward to implement by mapping grammar rules to functions.
*   **Expression Parsing**: A **Pratt Parser** (Top-Down Operator Precedence parser) will be implemented for parsing expressions. This method elegantly handles operator precedence and associativity, simplifying expression parsing logic considerably compared to traditional recursive descent for expressions.
*   **Indentation vs. Braces**: The parser will consume `INDENT`, `DEDENT`, and `NEWLINE` tokens from the lexer, along with `{` and `}` tokens, to correctly interpret `Block` structures as defined in the grammar (allowing either style per block).
*   **GLR Fallback**: As noted in `docs/Steps.md`, a GLR (Generalized LR) parser or similar advanced technique is considered a *fallback* strategy. With the lexer emitting `INDENT`/`DENT` tokens, many ambiguities related to optional significant indentation should be resolvable by the recursive descent / Pratt parser. GLR might only be necessary if other, more complex ambiguities arise that cannot be easily handled by LL(k) lookahead or the chosen primary strategies.
*   **Implementation Language**: The parser will be implemented in Rust.
*   **AST Node Allocation**: For performance, AST nodes will typically be allocated using an arena allocator (e.g., a structure like `ast::Arena` or a library like `bumpalo`). This avoids frequent small heap allocations and deallocations. Details will be in `AST_SPECIFICATION.md`.

## 3. Token Handling

*   **Token Stream**: The parser will operate on a token stream provided by the lexer. This might be a buffered stream or an iterator.
*   **Token Consumption**: The parser will typically consume tokens one at a time (`peek()` and `consume()` operations).
*   **Lookahead**: Limited lookahead (e.g., LL(1) or LL(k) where k is small) will be used to decide between alternative grammar productions.
*   **Whitespace & Comments**: The lexer is expected to have already processed/skipped most whitespace and comments. The parser will primarily see meaningful tokens, including `NEWLINE`, `INDENT`, and `DEDENT` where relevant.
*   **EOF Sentinel**: The lexer is expected to always append an `EOF` (End-Of-File) sentinel token to the stream. The parser can loop consuming tokens until `EOF` is encountered.

## 4. Parsing Declarations

(Based on `RFC-001_SYNTAX_GRAMMAR.md`, Section 2)

*   **`VariableDecl` (`let`, `var`)**: Parse the keyword, identifier, optional type annotation (`:` Type), initializer (`=` Expression), and optional statement terminator.
*   **`FunctionDecl` (`fn`, `async fn`)**: Parse optional `async` keyword, `fn` keyword, function name (IDENTIFIER), parameter list (`(` ParameterList? `)`), optional return type (`->` Type), and function body (Block).
*   **`DataClassDecl` (`data`)**: Parse `data` keyword, data class name (IDENTIFIER), and field list (`{` FieldList? `}`).

## 5. Parsing Statements

(Based on `RFC-001_SYNTAX_GRAMMAR.md`, Section 5)

*   **`LetDeclarationStatement`**: Parsed as a `VariableDecl`.
*   **`ExpressionStatement`**: Parse an `Expression` followed by an optional statement terminator. Note: A bare `NEWLINE` token occurring within parentheses `()` that are part of an expression being parsed (e.g., a function call with arguments spanning multiple lines) does **not** terminate the `ExpressionStatement`; newline termination rules apply at the statement level after the full primary expression is parsed.
*   **`BlockStatement`**: Parse a `Block` (either `BraceBlock` or `IndentedBlock`).
*   **`ReturnStatement`**: Parse `return` keyword, optional `Expression`, and optional statement terminator.
*   **`IfStatement`**: Parse `if` keyword, condition `Expression`, `Block` for the true-branch, and an optional `else` branch (which can be another `Block` or a chained `IfStatement`).
*   **`WhileStatement`**: Parse `while` keyword, condition `Expression`, and a `Block` for the loop body.
*   **`ForStatement`**: Parse `for` keyword, loop variable `IDENTIFIER`, `in` keyword, iterable `Expression`, and a `Block` for the loop body.
*   **`BreakStatement`**: Parse `break` keyword and optional statement terminator.
*   **`ContinueStatement`**: Parse `continue` keyword and optional statement terminator.
*   **Statement Termination**: The parser will implement the newline/semicolon termination logic as described in `RFC-001_SYNTAX_GRAMMAR.md` (§1.1 and §5), using `NEWLINE` tokens and context to determine when a statement ends.

## 6. Parsing Expressions (Pratt Parser)

(Based on `RFC-001_SYNTAX_GRAMMAR.md`, Section 4 and Appendix A. The operator precedence and associativity rules are detailed in `RFC-001_SYNTAX_GRAMMAR.md`, Appendix A, Table A-1, levels 15 down to 1.)

*   **Core Idea**: Each token type that can be part of an expression is associated with a precedence value and parsing functions (handlers).
    *   **NUD (Null Denotation)**: Called for tokens that appear at the beginning of an expression (e.g., literals, identifiers, prefix operators).
    *   **LED (Left Denotation)**: Called for tokens that appear in an infix or postfix position (e.g., binary operators, function calls, postfix operators).
*   **Operator Precedence & Associativity**: The parser will use the table defined in `RFC-001_SYNTAX_GRAMMAR.md`, Appendix A, to manage operator precedence and associativity.
*   **Atoms / Primary Expressions**: The base cases for expressions, parsed by NUD handlers:
    *   `Literal` (String, Integer, Float, Boolean `true`/`false`).
    *   `IDENTIFIER` (variable access).
    *   `QualifiedIdentifier`.
    *   `GroupedExpr` (`(` Expression `)`).
    *   `IfExpression` (if `if` is used as an expression).
    *   `MatchExpr`.
    *   Array, Tuple, Map literals (⚠️ **TBD (AST-1)** syntax for these literals).
*   **Unary Operators**: Handled by prefix NUD handlers (e.g., `!`, `-`, `+`).
*   **Binary Operators**: Handled by infix LED handlers (e.g., `+`, `-`, `*`, `/`, `&&`, `==`, `??`).
*   **Postfix Operators**: Handled by postfix LED handlers (e.g., `?` error propagation, `.await`).
*   **Function Calls & Member Access**: `()` and `.` are typically handled as high-precedence infix (LED) operations.
*   **Future Operators**: Stubs or considerations for future operators like the pipeline operator `|>` (⚠️ **TBD (PARSE-PIPE)**) would be added as LED handlers if adopted.

## 7. Parsing Types

(Based on `RFC-001_SYNTAX_GRAMMAR.md`, Section 3)

Parser functions will be responsible for recognizing and constructing AST nodes for type expressions:
*   `IDENTIFIER` (simple type name).
*   `QualifiedIdentifier`.
*   `TupleType` (`(` ... `)`).
*   `ArrayType` (`[` Type `]`).
*   `FunctionType` (`fn(` ... `) ->` ...).
*   `GenericType` (`IDENTIFIER <` ... `>`).
*   (⚠️ **TBD (PARSE-NULLABLE-TYPE)**: If nullable types are introduced with a specific syntax like `Type?` (distinct from `Option<Type>`), the parser would need to handle this optional postfix `?` in type expressions.)

## 8. Block Structure Parsing

(Based on `RFC-001_SYNTAX_GRAMMAR.md`, Section 5, and indentation rules in Section 1.1)

*   The parser will expect either a `{` (for `BraceBlock`) or an `INDENT` token (for `IndentedBlock`) when a `Block` is grammatically required.
*   **`BraceBlock`**: Consume `{`, parse zero or more `Statement`s until `}` is found. The `}` token is consumed.
*   **`IndentedBlock`**: Consume `INDENT`, parse one or more `Statement`s until a matching `DEDENT` token is found. The `DEDENT` token is consumed.
*   **Hygiene Rule**: The parser (or a subsequent semantic check) must enforce that a single block does not mix brace-style and indentation-style for its immediate structure (e.g., cannot open with `{` and close with `DEDENT`).
*   **Single-Statement Shortcuts**: For constructs like `if`, `while`, `for`, if a `Block` is expected but neither `{` nor `INDENT` follows, the parser will attempt to parse the very next single `Statement` as the body, as per `RFC-001` (Rule 4 of the indentation proposal).

    *Example Pseudo-code for Block Styles:*
    ```ferra
    // Brace Style
    if condition {
        do_one_thing();
        do_another_thing();
    }

    // Indentation Style
    if condition:
        INDENT
        do_one_thing()
        do_another_thing()
        DEDENT

    // Single Statement Shortcut (after if/while/for etc.)
    if condition do_the_only_thing()
    ```
    (* Note: `INDENT`/`DEDENT` are conceptual tokens from the lexer. *)

## 9. Error Handling & Recovery

*   **Error Reporting**: When a syntax error is encountered (e.g., unexpected token), the parser will generate an error message.
    *   Messages **MUST** include precise source location information (from tokens).
    *   Messages **SHOULD** strive for the "positive-first" style: explain what was expected or valid, then what was found, and offer actionable suggestions if possible.
*   **Recovery**: To find multiple errors in a single pass, the parser might attempt error recovery.
    *   Common strategies: Panic mode (skip tokens until a synchronizing token is found). Synchronizing tokens typically include statement terminators like `;`, block closers like `}` or `DEDENT`, and keywords that unambiguously start new declarations or major statements (e.g., `fn`, `data`, `let`, `if`, `while`, `for`, `match`).
    *   More advanced recovery (* ... *)
    *   ⚠️ **TBD (PARSE-1)**: Define specific error recovery strategies.

## 10. AST Node Generation (Overview)

*   As the parser successfully recognizes grammar rules, it will construct nodes for an Abstract Syntax Tree (AST).
*   Each AST node will represent a language construct (e.g., `VariableDeclarationNode`, `FunctionCallNode`, `IfStatementNode`).
*   Nodes will store relevant information, such as identifiers, operators, sub-expressions, child statements, and source location spans.
*   AST nodes are typically stored in an arena (e.g., `ast::Arena` or similar) for efficient allocation (see `AST_SPECIFICATION.md`, §2 for details on allocation strategy).
*   The detailed structure of AST nodes will be defined in `AST_SPECIFICATION.md` (Step 1.1.3).

## 11. Parser API (Conceptual)

A typical entry point for the parser might look like:

```ferra // Conceptual Ferra-like or Rust signature
fn parse_compilation_unit(tokens: TokenStream) -> Result<ast::CompilationUnit, Vec<ParseError>>

// Or for parsing a single expression (e.g., for a REPL or macro context)
fn parse_expression(tokens: TokenStream) -> Result<ast::Expression, Vec<ParseError>>

// Convenience wrapper for parsing a whole file
fn parse_file(path: Path) -> Result<ast::CompilationUnit, Vec<ParseError>> {
    // 1. Read file content from Path
    // 2. Lex content into TokenStream
    // 3. Call parse_compilation_unit(token_stream)
    // Handle I/O and lexing errors appropriately
}
```
Where `TokenStream` is the output from the lexer, `ast::CompilationUnit` and `ast::Expression` are types representing the root of the AST (or an expression subtree), and `ParseError` is a struct/enum detailing syntax errors.

## 12. Open Questions & Future Considerations (Parser Specific)

*   **(PARSE-1)** Specific error recovery strategies and quality of diagnostic messages under various error conditions.
*   **(PARSE-AMBIGUITY)** Handling of parser-level ambiguities if the lexer's `INDENT`/`DEDENT` strategy isn't sufficient (i.e., when/if GLR or backtracking might truly be needed).
*   **(PARSE-PERF)** Performance of the Pratt parser for deeply nested or complex expressions.
*   **(PARSE-CONTEXT)** Parser state management for contextual keywords or complex lookahead decisions (if any arise beyond simple LL(k)).
*   **(PARSE-MACRO)** Integration with macro expansion (how macro results are re-parsed or spliced into the AST).
*   **(PARSE-CONDCOMP)** Support for conditional compilation if it affects parsing.
*   **(PARSE-PIPE)** Handling of pipeline operator `|>` if added in the future.
*   **(PARSE-NULLABLE-TYPE)** Parsing of specific nullable type syntax (e.g. `Type?`) if adopted. 