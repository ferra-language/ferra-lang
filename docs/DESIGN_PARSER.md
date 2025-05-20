# Ferra Parser Design v0.1

This document specifies the design for the Ferra v0.1 parser. The parser is the second stage of the Ferra compiler front-end, following the lexer. It takes a stream of tokens (from `DESIGN_LEXER.md`) as input and produces an Abstract Syntax Tree (AST) representing the syntactic structure of the source code, based on `docs/SYNTAX_GRAMMAR_V0.1.md`.

## 1. Introduction & Goals

*   **Purpose**: To analyze the token stream for syntactic correctness according to the Ferra grammar and to construct an AST that accurately reflects the code's structure.
*   **Input**: A stream of tokens from the Ferra Lexer.
*   **Output**: An Abstract Syntax Tree (AST). If parsing fails, a list of syntax errors with source location information.
*   **Grammar Adherence**: Strictly follow `docs/SYNTAX_GRAMMAR_V0.1.md`.
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

(Based on `docs/SYNTAX_GRAMMAR_V0.1.md`, Section 2)

*   **`VariableDecl` (`let`, `var`)**: Parse the keyword, identifier, optional type annotation (`:` Type), initializer (`=` Expression), and optional statement terminator.
*   **`FunctionDecl` (`fn`, `async fn`, `extern "C" fn`)**: 
    *   Parse optional `AttributeListOpt`.
    *   Parse optional visibility (e.g., `pub`).
    *   Parse optional `unsafe` keyword.
    *   Parse optional `async` keyword.
    *   Parse optional `extern AbiStringLiteral` (e.g., `extern "C"`) for specifying linkage/calling convention.
    *   Parse `fn` keyword, function name (IDENTIFIER).
    *   Parse `ParameterList` (`(` Parameter (` ,` Parameter)* `)? `)`). Each `Parameter` also parses an `AttributeListOpt`.
    *   Parse optional return type (`->` Type).
    *   Parse either a `Block` for the function body or a terminating `;` (for declarations without a Ferra body, common in FFI or forward declarations).
    *   Construct a `FunctionDeclNode` in the AST, including any attributes, linkage info, and body/terminator.
*   **`DataClassDecl` (`data`)**: 
    *   Parse optional `AttributeListOpt`.
    *   Parse `data` keyword, data class name (IDENTIFIER).
    *   Parse field list (`{` FieldList? `}`). Each `Field` in `FieldList` also parses an `AttributeListOpt`.
    *   Construct a `DataClassDeclNode` in the AST with attributes and fields.

*   **Attribute Handling Note**: The parser is responsible for consuming `AttributeListOpt` where specified in the grammar (e.g., for `FunctionDecl`, `DataClassDecl`, `ExternBlock`, `ExternFunctionDecl`, `ExternVariableDecl`, `Parameter`, `Field`) and attaching the list of parsed `AttributeNode`s to the corresponding declaration node in the AST. Semantic validation of attributes occurs later.

*   **`ExternBlock` (FFI)**:
    *   (Based on `docs/SYNTAX_GRAMMAR_V0.1.md`, Section 2.4)
    *   Parse optional `AttributeListOpt`.
    *   Parse the `extern` keyword.
    *   Parse the `AbiStringLiteral` (a `StringLiteral` token, e.g., `"C"`). Semantic validation of the ABI string occurs later.
    *   Parse the opening `{`.
    *   Parse zero or more `ExternalItem`s until `}` is found:
        *   **`ExternFunctionDecl`**: 
            *   Parse optional `AttributeListOpt`.
            *   Parse optional `async` keyword (though typically not used for C FFI).
            *   Parse `fn` keyword, function name (IDENTIFIER).
            *   Parse `ParameterList` (`(` Parameter (` ,` Parameter)* `)? `)`).
            *   Parse optional return type (`->` Type).
            *   Parse the terminating `;`.
            *   Construct an `ExternFunctionDeclNode` in the AST.
        *   **`ExternVariableDecl`**:
            *   Parse optional `AttributeListOpt`.
            *   Parse `static` keyword, variable name (IDENTIFIER).
            *   Parse type annotation (`:` Type).
            *   Parse the terminating `;`.
            *   Construct an `ExternVariableDeclNode` in the AST.
    *   Parse the closing `}`.
    *   Construct an `ExternBlockNode` in the AST, containing the list of external items and the ABI string.

## 5. Parsing Statements

(Based on `docs/SYNTAX_GRAMMAR_V0.1.md`, Section 5)

*   **`LetDeclarationStatement`**: Parsed as a `VariableDecl`.
*   **`ExpressionStatement`**: Parse an `Expression` followed by an optional statement terminator. Note: A bare `NEWLINE` token occurring within parentheses `()` that are part of an expression being parsed (e.g., a function call with arguments spanning multiple lines) does **not** terminate the `ExpressionStatement`; newline termination rules apply at the statement level after the full primary expression is parsed.
*   **`BlockStatement`**: Parse a `Block` (either `BraceBlock` or `IndentedBlock`).
*   **`ReturnStatement`**: Parse `return` keyword, optional `Expression`, and optional statement terminator.
*   **`IfStatement`**: Parse `if` keyword, condition `Expression`, `Block` for the true-branch, and an optional `else` branch (which can be another `Block` or a chained `IfStatement`).
*   **`WhileStatement`**: Parse `while` keyword, condition `Expression`, and a `Block` for the loop body.
*   **`ForStatement`**: Parse `for` keyword, loop variable `IDENTIFIER`, `in` keyword, iterable `Expression`, and a `Block` for the loop body.
*   **`BreakStatement`**: Parse `break` keyword and optional statement terminator.
*   **`ContinueStatement`**: Parse `continue` keyword and optional statement terminator.
*   **Statement Termination**: The parser will implement the newline/semicolon termination logic as described in `docs/SYNTAX_GRAMMAR_V0.1.md` (§1.1 and §5), using `NEWLINE` tokens and context to determine when a statement ends.

## 5. Expression Parsing (Pratt Parser)

Ferra will employ a Pratt parser (Top-Down Operator Precedence parser) for its expression parsing. This method is well-suited for handling expressions with infix, prefix, and postfix operators, providing an elegant way to manage varying precedence levels and associativity.

### Operator Precedence and Associativity

The binding powers (for left-binding and null-denotation) and the associativity rules (left, right, or non-associative) for all operators handled by the Pratt parser will be derived directly from the canonical operator precedence table defined in **`docs/SYNTAX_GRAMMAR_V0.1.md`, Appendix A**. This ensures that the parser's behavior is perfectly synchronized with the language's grammatical specification. Implementers will translate this table into the necessary functions (NUDs and LEDs) and precedence values for the Pratt parsing engine.

*Note on `and`/`or` keywords: The lexer (`DESIGN_LEXER.md`) normalizes the `and` and `or` keywords into `&&` and `||` tokens respectively. The parser therefore only needs to define behavior for the `&&` and `||` tokens regarding logical operations.*

*   **Core Idea**: Each token type that can be part of an expression is associated with a precedence value and parsing functions (handlers).
    *   **NUD (Null Denotation)**: Called for tokens that appear at the beginning of an expression (e.g., literals, identifiers, prefix operators).
    *   **LED (Left Denotation)**: Called for tokens that appear in an infix or postfix position (e.g., binary operators, function calls, postfix operators).
*   **Atoms / Primary Expressions**: The base cases for expressions, parsed by NUD handlers:
    *   `Literal` (String, Integer, Float, Boolean `true`/`false`).
    *   `IDENTIFIER` (variable access).
    *   `QualifiedIdentifier`.
    *   `GroupedExpr` (`(` Expression `)`).
    *   `IfExpression` (if `if` is used as an expression).
    *   `MatchExpr`. (See details below on Match Expression and Pattern Parsing).
    *   Array, Tuple, Map literals (⚠️ **TBD (AST-1)** syntax for these literals).
*   **Unary Operators**: Handled by prefix NUD handlers (e.g., `!`, `-`, `+`).
*   **Binary Operators**: Handled by infix LED handlers (e.g., `+`, `-`, `*`, `/`, `&&`, `==`, `??`).
*   **Postfix Operators**: Handled by postfix LED handlers (e.g., `?` error propagation, `.await`).
*   **Function Calls & Member Access**: `()` and `.` are typically handled as high-precedence infix (LED) operations.
*   **Future Operators**: Stubs or considerations for future operators like the pipeline operator `|>` (⚠️ **TBD (PARSE-PIPE)**) would be added as LED handlers if adopted.

### Match Expression and Pattern Parsing

Parsing `MatchExpr ::= "match" Expression "{" (MatchArm)+ "}"` involves:
1.  Parsing the scrutinee `Expression`.
2.  Parsing one or more `MatchArm`s within the braces.

Each `MatchArm ::= Pattern "=>" Expression (";" | ",")?` requires dedicated pattern parsing logic.

**Pattern Parsing (`Pattern` from `SYNTAX_GRAMMAR_V0.1.md`):**
The parser will implement functions to parse the various forms of `Pattern`:

*   **`Literal` Pattern**: Matches literal tokens directly (e.g., `123`, `"hello"`, `true`). The parser consumes the literal token and builds a literal pattern AST node.
*   **`IDENTIFIER` Pattern (Binding)**: 
    *   If an `IDENTIFIER` is not followed by `{` (for data class pattern) and is not a known constant/enum variant without data, it's treated as a variable binding. 
    *   The parser creates a binding pattern AST node that will capture the matched value.
    *   (Optional) `IDENTIFIER @ Pattern` syntax (if added later) for binding a sub-pattern to a name would require parsing this specific sequence.
*   **`_` (Wildcard Pattern)**: Consumes the `_` token and creates a wildcard pattern AST node.
*   **`DataClassPattern`**: `IDENTIFIER "{" (FieldPattern ("," FieldPattern)* (",")? )? (".."?)? "}"`
    1.  Parse the leading `IDENTIFIER` (the data class name).
    2.  Parse the opening `{`.
    3.  Parse a comma-separated list of `FieldPattern`s:
        *   **`FieldPattern ::= IDENTIFIER (":" Pattern)?`**: Parse the field name `IDENTIFIER`. If a `:` follows, recursively parse the sub-`Pattern` for that field.
        *   **`FieldPattern ::= IDENTIFIER` (Shorthand)**: Parse the field name `IDENTIFIER`. This is shorthand for `IDENTIFIER : IDENTIFIER`, binding the field's value to a variable of the same name.
    4.  Optionally parse `..` (rest pattern) if present.
    5.  Parse the closing `}`.
    6.  Construct a data class pattern AST node with the class name, field patterns, and rest indicator.
*   **Tuple Pattern `(Pattern, ...)`** (If tuple data structures are directly destructure-able in `match` beyond simple IDENTIFIER binding for a tuple value):
    *   Parse `(`, then a comma-separated list of `Pattern`s, then `)`. This would align if `data EnumName { Variant1(Type1, Type2) }` variants are matched positionally.
*   **Other Patterns (Future)**: Placeholder for future extensions like range patterns, array patterns, etc.

The parser must be able to distinguish these pattern forms based on the token stream, often using one token of lookahead (e.g., to see if an `IDENTIFIER` is followed by `{` to start a `DataClassPattern`).

## 6. Parsing Types

(Based on `docs/SYNTAX_GRAMMAR_V0.1.md`, Section 3)

Parser functions will be responsible for recognizing and constructing AST nodes for type expressions:
*   `IDENTIFIER` (simple type name).
*   `QualifiedIdentifier`.
*   `TupleType` (`(` ... `)`).
*   `ArrayType` (`[` Type `]`).
*   `FunctionType` (`fn(` ... `) ->` ...).
*   `ExternFunctionType` (`extern "C" fn(` ... `) -> ` ...) (as per `SYNTAX_GRAMMAR_V0.1.md`).
*   `RawPointerType` (`*const Type`, `*mut Type`) (as per `SYNTAX_GRAMMAR_V0.1.md`).
*   `GenericType` (`IDENTIFIER <` ... `>`).
*   (⚠️ **TBD (PARSE-NULLABLE-TYPE)**: If nullable types are introduced with a specific syntax like `Type?` (distinct from `Option<Type>`), the parser would need to handle this optional postfix `?` in type expressions.)

## 7. Block Structure Parsing

(Based on `docs/SYNTAX_GRAMMAR_V0.1.md`, Section 5, and indentation rules in Section 1.1)

*   The parser will expect either a `{` (for `BraceBlock`) or an `INDENT` token (for `IndentedBlock`) when a `Block` is grammatically required.
*   **`BraceBlock`**: Consume `{`, parse zero or more `Statement`s until `}` is found. The `}` token is consumed.
*   **`IndentedBlock`**: Consume `INDENT`, parse one or more `Statement`s until a matching `DEDENT` token is found. The `DEDENT` token is consumed.
*   **Hygiene Rule**: The parser (or a subsequent semantic check) must enforce that a single block does not mix brace-style and indentation-style for its immediate structure (e.g., cannot open with `{` and close with `DEDENT`).
*   **Single-Statement Shortcuts**: For constructs like `if`, `while`, `for`, if a `Block` is expected but neither `{` nor `INDENT` follows, the parser will attempt to parse the very next single `Statement` as the body, as per `docs/SYNTAX_GRAMMAR_V0.1.md` (Rule 4 of the indentation proposal, or similar text).

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

## 8. Error Handling & Recovery

*   **Error Reporting**: When a syntax error is encountered (e.g., unexpected token), the parser will generate an error message.
    *   Messages **MUST** include precise source location information (from tokens).
    *   Messages **SHOULD** strive for the "positive-first" style: explain what was expected or valid, then what was found, and offer actionable suggestions if possible.
*   **Recovery Strategy (v0.1)**: For the initial v0.1 parser, the primary error recovery technique will be **panic mode**. Upon encountering an unexpected token, the parser will report the error and then attempt to synchronize by discarding tokens until it finds a token that can reliably start a new statement or declaration. 
    *   **Synchronizing Tokens**: This set typically includes statement terminators like `;`, block closers like `}` or `DEDENT`, and keywords that unambiguously start new declarations or major statements (e.g., `fn`, `data`, `let`, `var`, `if`, `while`, `for`, `match`, `return`).
    *   This approach allows parsing to continue beyond the first error to potentially identify further syntax issues in a single pass.
    *   More advanced recovery techniques (e.g., context-sensitive recovery, insertion/deletion of tokens) may be explored in future versions.
*   (PARSE-1 TBD entry can be removed or updated to reflect that a basic strategy is defined, e.g., "Further refinement of synchronizing token sets and specific recovery heuristics beyond basic panic mode.")

## 9. AST Node Generation (Overview)

*   As the parser successfully recognizes grammar rules, it will construct nodes for an Abstract Syntax Tree (AST).
*   Each AST node will represent a language construct (e.g., `VariableDeclarationNode`, `FunctionCallNode`, `IfStatementNode`).
*   Nodes will store relevant information, such as identifiers, operators, sub-expressions, child statements, and source location spans.
*   AST nodes are typically stored in an arena (e.g., `ast::Arena` or similar) for efficient allocation (see `AST_SPECIFICATION.md`, §2 for details on allocation strategy).
*   The detailed structure of AST nodes will be defined in `AST_SPECIFICATION.md` (Step 1.1.3).

## 10. Parser API (Conceptual)

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

## 11. Open Questions & Future Considerations (Parser Specific)

*   **(PARSE-1)** Further refinement of synchronizing token sets and specific recovery heuristics beyond basic panic mode for v0.1.
*   **(PARSE-AMBIGUITY)** Handling of parser-level ambiguities if the lexer's `INDENT`/`DEDENT` strategy isn't sufficient (i.e., when/if GLR or backtracking might truly be needed).
*   **(PARSE-PERF)** Performance of the Pratt parser for deeply nested or complex expressions.
*   **(PARSE-CONTEXT)** Parser state management for contextual keywords or complex lookahead decisions (if any arise beyond simple LL(k)).
*   **(PARSE-MACRO)** Integration with macro expansion (how macro results are re-parsed or spliced into the AST).
*   **(PARSE-CONDCOMP)** Support for conditional compilation if it affects parsing.
*   **(PARSE-PIPE)** Handling of pipeline operator `|>` if added in the future.
*   **(PARSE-NULLABLE-TYPE)** Parsing of specific nullable type syntax (e.g. `Type?`) if adopted. 