# Ferra Grammar v0.1 (draft)

This document defines the initial version (v0.1) of the Ferra programming language syntax using Extended Backus-Naur Form (EBNF).

## 0. Notation & Goals

*   EBNF syntax (as defined below), targeting Unicode code-points for terminals where applicable.
*   The grammar aims to be suitable for a parser leveraging Pratt parsing for expressions.
*   A GLR (or similar) parsing strategy may be employed as a fallback or for specific ambiguous constructs, particularly concerning optional significant indentation.

**EBNF Conventions Used:**

*   `::=`        : Definition
*   `|`          : Alternative
*   `?`          : Optional (zero or one occurrence) (*Note: also used as `[]` in the user suggestion, I will stick to `?` or `[...]` consistently*)
*   `*`          : Zero or more occurrences
*   `+`          : One or more occurrences
*   `()`         : Grouping
*   `"terminal"` : Literal terminal symbols (keywords, operators, punctuation)
*   `UPPERCASE_NAME` : Abstract terminal symbols (e.g., IDENTIFIER, INTEGER_LITERAL, typically defined by lexer rules).
*   `(* ... *)`  : EBNF-style comments (not part of Ferra syntax).

Whitespace (spaces, tabs) is generally insignificant unless part of significant indentation (covered later) or within string literals. Newlines often terminate statements or are significant in certain contexts.

---

## 1. Lexical Structure

  (* This section details the low-level lexical elements of Ferra. These are typically processed by a lexer (tokenizer) before the parser constructs the Abstract Syntax Tree. *)

  1.1 Whitespace & Significant Indentation

      (* General Whitespace *)
      (*   Whitespace characters such as space (U+0020), horizontal tab (U+0009),
           are generally insignificant between tokens. They serve to separate tokens.
           Multiple whitespace characters are usually treated as one.
      *)

      (* Newlines & Statement Termination *)
      (*   The Ferra language is newline-sensitive for statement termination, aiming for a balance
           between Python-like readability and the explicitness available in languages like Rust or Swift.
      *)
      NEWLINE ::= (* Represents one or more platform-agnostic newline characters. The lexer emits a raw NEWLINE token (or equivalent) upon encountering a physical line break not within a multi-line construct like a block comment or multi-line string literal. *)

      (* General Rules for Newlines as Statement Terminators:
         1. A physical NEWLINE token, after lexing, generally signals the end of a statement.
         2. Explicit Semicolon: A semicolon `;` always terminates a statement. It can be used to separate multiple statements on a single line.
         3. Suppression of NEWLINE termination: A NEWLINE token does NOT terminate a statement if the preceding token on the line indicates incompleteness.
            The parser, when deciding if a NEWLINE acts as a terminator, will consider the preceding token.
            A line is considered "clearly incomplete" if the last token before the NEWLINE is one of:
              - An opening delimiter: `(`, `[`, `{` (when its corresponding closer has not yet been seen at the same nesting level).
              - A comma `,`.
              - A dot `.` (for member access).
              - An infix or prefix operator that typically expects a subsequent operand (e.g., `+`, `-`, `*`, `/`, `%`, `<<`, `>>`, `&`, `^`, `|`, `&&`, `||`, `??`, `=`, `+=`, etc., and unary `!`, `-`, `+`).
              - The `->` arrow for function return types.
              - The error propagation operator `?` when used in a postfix position.
              - The `async` or `fn` keywords if a function signature is incomplete.
              - The `let` or `var` keywords if a variable declaration is incomplete.
              - The `data` keyword if a data declaration is incomplete.
              - The `match` keyword if the expression or block is incomplete.
         4. Closing Brace: Inside `{ ... }` blocks, the closing brace `}` also implicitly terminates the last statement within the block if that statement isn't already terminated by a newline or semicolon.
         5. Explicit Line Continuation: (Deferred for now) A backslash `\` at the very end of a line could be introduced later if needed for explicit line continuation, but is not part of the v0.1 grammar.

         The parser is responsible for disambiguating whether a NEWLINE acts as a statement terminator or as insignificant whitespace based on these rules.
         This allows for code like:
           let total = a
               + b      // NEWLINE after `+` is suppressed
               + c      // NEWLINE after `+` is suppressed

           fetch(url)        // NEWLINE after `)` is suppressed if part of a larger expression/chain
               .await?      // NEWLINE after `?` is suppressed
               .json()

           let x = 1; let y = 2 // Explicit semicolons
      *)

      (* Significant Indentation *)
      (*   Ferra supports significant indentation for defining blocks of code, offering a Python-like alternative
           to explicit curly braces. The lexer **MUST** track indentation levels at the start of
           each logical line (i.e., after handling line continuations if any, though explicit line
           continuation characters like `\` are deferred for v0.1).

           - An `INDENT` token is emitted when a new logical line has a greater indentation level
             than the previous logical line (and is not empty or a comment-only line).
           - A `DEDENT` token is emitted when a new logical line has a lesser indentation level.
             Multiple `DEDENT` tokens may be emitted if the indentation decreases by several levels.
           - `NEWLINE` tokens are still emitted as usual at the end of logical lines. The parser will
             use these in conjunction with `INDENT`/`DEDENT` and the statement termination rules.
           - Empty lines or lines containing only whitespace and/or comments do not affect the current
             indentation level for token emission purposes.
           - The exact number of spaces or tabs constituting an indentation level should be consistent
             within a given indented block. Mixing tabs and spaces for indentation at the same level
             is typically an error or discouraged.

           This mechanism allows `Block` structures to be defined either by braces or by indentation.
      *)

      (* Style Hygiene & Error Diagnostics for Indentation and Braces *)
      (*
         While Ferra offers flexibility in block structuring, specific rules prevent ambiguity and promote clarity:
         1.  **Exclusive Choice per Block**: A single syntactic block **MUST** use either curly braces `{...}` OR significant
             indentation. It is a compile-time error to mix these for the *same* block (e.g., opening with `{`
             and then relying on dedentation to close it, or vice-versa).
             Example Error: `if condition { indent_statement_1 // ERROR: Mixed style`

         2.  **Consistent Indentation**: Within an `IndentedBlock`, all statements **MUST** be at the same increased
             indentation level relative to the line that introduced the block. Inconsistent indentation within
             the same logical block level is an error.
             Example Error: `fn foo():
                              INDENT
                                  let x = 1
                                let y = 2 // ERROR: Inconsistent indentation for statement y`

         3.  **Expected Indentation**: If a construct implies a block but no `{` is found (and the syntax allows
             an indented block), an `INDENT` token (or a statement at an increased indent level) is expected.
             Failure to indent will result in a compile-time error.
             Example Error: `fn bar():
                            let x = 1 // ERROR: Expected indented block or single statement on same line after definition`

         Nested blocks can independently choose their style (braces or indentation).
         A code formatter (`ferrafmt`) will have a canonical style (e.g., braces for multi-line blocks by default)
         and may offer options to convert between styles.
      *)

  1.2 Comments
      ```ebnf
      LineComment  ::= "//" (~NEWLINE)* (NEWLINE | EOF)
      BlockComment ::= "/*" ( BlockCommentContent )* "*/"
      BlockCommentContent ::= BlockComment (* for nesting *)
                            | (~("*/") AnyCharacter) (* any character not part of "*/" *)

      (* AnyCharacter represents any valid Unicode character.
         The definition above allows for nested block comments, similar to Rust.
         Both comment types are typically treated as whitespace by the parser. *)
      ```

  1.3 Tokens (Keywords, Identifiers, Literals, Punctuation)

      1.3.1 Keywords
          ```ebnf
          KEYWORD ::= "let"       (* Variable declaration, immutable *)
                    | "var"       (* Variable declaration, mutable *)
                    | "fn"        (* Function definition *)
                    | "async"     (* Asynchronous function modifier *)
                    | "data"      (* Data class definition *)
                    | "match"     (* Pattern matching expression/statement start *)
                    | "true"      (* Boolean literal *)
                    | "false"     (* Boolean literal *)
                    | "and"       (* Logical AND (alias for &&) *)
                    | "or"        (* Logical OR (alias for ||) *)
                    | "extern"    (* External function/variable block specifier *)
                    | "return"    (* Return statement *)
                    | "if"        (* Conditional statement *)
                    | "else"      (* Conditional alternative *)
                    | "while"     (* While loop *)
                    | "for"       (* For loop *)
                    | "in"        (* For loop iterator keyword *)
                    | "break"     (* Break from loop *)
                    | "continue"  (* Continue loop iteration *)
                    | "pub"       (* Public visibility modifier *)
                    | "unsafe"    (* Unsafe operation marker *)
                  (*| "loop"    | "import"  | "export"*)
                  (*| "type"    | "static"  | "const"   | "super" *)
                  (*| "self"    | "Self"    | "crate"   | "mod"     | "use"   *)
                  (*| "where"   | "impl"    | "trait" *)
                  (*| "enum"    | "struct"  | "union"   | "yield" *)
                  (*| ... other keywords to be added as features are defined ... *)
          ```
          (* Note: The lexer should treat `and` as `&&` and `or` as `||` for the parser. *)

      1.3.2 Identifiers
          ```ebnf
          IDENTIFIER ::= ID_START (ID_CONTINUE)*
          (*
             ID_START and ID_CONTINUE are character classes defined by the Unicode
             Standard Annex #31, "Unicode Identifier and Pattern Syntax".
             This allows for a wide range of Unicode characters in identifiers,
             promoting internationalization.
             Keywords are typically reserved and cannot be used as IDENTIFIERs.
          *)
          ```

      1.3.3 Literals
          ```ebnf
          LITERAL ::= StringLiteral
                    | IntegerLiteral
                    | FloatLiteral
                    | BooleanLiteral
                    | CharacterLiteral

          StringLiteral ::= StandardStringLiteral (* | RawStringLiteral | MultiLineStringLiteral *)

          StandardStringLiteral ::=
              '"' ( StringCharacter | SimpleEscapeSequence | UnicodeEscapeSequence )* '"'

          StringCharacter ::= ~['"' '\\' NEWLINE] (* Any character except quote, backslash, or raw newline *)

          SimpleEscapeSequence ::=
              "\\"  (* Literal backslash \\ *)
            | "\""  (* Literal double quote \" *)
            | "\'"  (* Literal single quote \' (if char literals are added) *)
            | "\n"   (* Line feed (LF) *)
            | "\r"   (* Carriage return (CR) *)
            | "\t"   (* Horizontal tab (HT) *)
            | "\0"   (* Null character (NUL) *)
          
          UnicodeEscapeSequence ::=
              "\u{" HEX_DIGIT_SEQUENCE "}" (* e.g., \u{7E}, \u{1F600} *)
          
          HEX_DIGIT_SEQUENCE ::= HEX_DIGIT (HEX_DIGIT (HEX_DIGIT (HEX_DIGIT (HEX_DIGIT HEX_DIGIT?)?)?)?)? (* 1 to 6 hex digits *)

          IntegerLiteral ::= DecimalIntegerLiteral
                           | HexIntegerLiteral
                           | OctalIntegerLiteral
                           | BinaryIntegerLiteral

          DecimalIntegerLiteral ::= DIGIT (DIGIT | "_")*
          HexIntegerLiteral     ::= "0x" HEX_DIGIT (HEX_DIGIT | "_")*
          OctalIntegerLiteral   ::= "0o" OCTAL_DIGIT (OCTAL_DIGIT | "_")*
          BinaryIntegerLiteral  ::= "0b" BINARY_DIGIT (BINARY_DIGIT | "_")*
          (* Note: Underscores are for visual separation and are ignored in the numeric value. *)
          (* Note: A leading zero alone does not typically imply octal by default to avoid confusion; explicit 0o is preferred. *)

          FloatLiteral ::=
              DIGIT (DIGIT | "_")* "." DIGIT (DIGIT | "_")* (ExponentPart)?
            | DIGIT (DIGIT | "_")* ExponentPart
            (* | "." DIGIT (DIGIT | "_")* (ExponentPart)? (* If leading dot floats are allowed *) *)
          ExponentPart ::= ("e" | "E") ("+" | "-")? DIGIT+
          (* Note: Underscores also allowed for visual separation in float literals. *)

          BooleanLiteral ::= "true" | "false"

          CharacterLiteral ::= "\'" ( StringCharacter | SimpleEscapeSequenceNoQuote | UnicodeEscapeSequence ) "\'"
          SimpleEscapeSequenceNoQuote ::= "\\" | "\n" | "\r" | "\t" | "\0" (* Excludes \' *)

          DIGIT        ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
          HEX_DIGIT    ::= DIGIT | "a" | "b" | "c" | "d" | "e" | "f"
                               | "A" | "B" | "C" | "D" | "E" | "F"
          OCTAL_DIGIT  ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7"
          BINARY_DIGIT ::= "0" | "1"
          ```

      1.3.4 Punctuation & Operators
          ```ebnf
          PUNCTUATION ::= ":"            (* Type annotation, field separator, map key-value *)
                        | "="            (* Assignment, default value *)
                        | "->"           (* Function return type arrow *)
                        | "{"            (* Block start, data class start, match block start *)
                        | "}"            (* Block end, data class end, match block end *)
                        | "("            (* Parameter list start, expression grouping, tuple start *)
                        | ")"            (* Parameter list end, expression grouping, tuple end *)
                        | ","            (* List separator (parameters, fields, arguments, array elements) *)
                        | "."            (* Member access (method call, field access), float point *)
                        | "=>"           (* Match arm separator *)
                        | "_"            (* Wildcard pattern, ignored identifier (sometimes) *)
                        | "!"            (* Macro invocation indicator, logical NOT (if unary op) *)
                        | "::"           (* Path separator (modules/namespaces) *)
                        | "<"            (* Less than operator, generic argument list start *)
                        | ">"            (* Greater than operator, generic argument list end *)
                        | ";"            (* Statement terminator (optional in many contexts) *)
                        | "["            (* Array/List type start, array literal start, index start *)
                        | "]"            (* Array/List type end, array literal end, index end *)
                        (* Binary Operators *)
                        | "+" | "-" | "*" | "/" | "%"  (* Arithmetic *)
                        | "==" | "!=" | "<" | "<=" | ">" | ">=" (* Comparison *)
                        | "&&" | "||" (* Logical AND, OR *)
                        | "&" | "|" | "^" | "<<" | ">>" (* Bitwise *)
                        | "??"           (* Nil-coalescing *)
                        | ".." | "..="    (* Range *)
                        (* Assignment Operators *)
                        | "=" | "+=" | "-=" | "*=" | "/=" | "%="
                        | "&=" | "|=" | "^=" | "<<=" | ">>="
                        (* Unary Operators (some also binary) *)
                        | "!"            (* Logical NOT, Macro invocation *)
                        (* Postfix Operators (special handling) *)
                        | "?"            (* Error propagation *)
                      (*| ... other punctuation/operators ... *)
          ```

      (* NEW SECTION FOR ATTRIBUTES *)
      1.3.5 Attributes (* General syntactic markers/annotations *)
          ```ebnf
          AttributeListOpt ::= (Attribute)*
          Attribute        ::= "#[" AttributeContent "]"
          (* Example: #[test], #[derive(Copy, Debug)], #[ai.assume(nll="noalias")], #[gpu], #[gpu(target_env="vulkan1.2")] *)
          (* Note: Inner attributes like `#![allow(dead_code)]` for module/crate scope are TBD if needed. *)

          AttributeContent   ::= AttributePath ( "(" AttributeArguments? ")" )? 
          AttributePath      ::= IDENTIFIER ( "." IDENTIFIER )*     (* e.g., test, derive, ai.assume, cfg.test *)
          AttributeArguments ::= Expression ( "," Expression)* (",")? 
                               (* Arguments are parsed as a list of expressions. 
                                  The specific structure expected (e.g., key=value, single literal) 
                                  is typically defined by the semantic rules of each attribute. *)
          ```

---

## 2. Declarations

  2.1 Variable Declaration
      ```ebnf
      VariableDecl ::= ("let" | "var") IDENTIFIER ( ":" Type )? "=" Expression ";"? (* Semicolon optional? TBD with statement termination rules*)
      (* Example: let pi: Float = 3.14159 *)
      (* Example: var count: Int = 0 *)
      ```

  2.2 Function Declaration
      ```ebnf
      FunctionDecl ::= AttributeListOpt ("pub")? ("unsafe")? ("async")? ("extern" AbiStringLiteral)? "fn" IDENTIFIER ParameterList ( "->" Type )? ( Block | ";" )
      (* Example: async fn fetch(url: String) -> Result<Response> { ... } *)
      (* Example: #[test] fn my_test_function() { ... } *)
      (* Example (exporting for FFI): pub extern "C" fn my_exported_c_func(p: *const c_char) -> c_int; *)
      (* Example (exporting for FFI with body): pub extern "C" fn my_ferra_func_for_c() -> i32 { return 42; } *)
      (* If Block is absent, a semicolon is required, indicating a forward declaration or an extern function without a Ferra body (if not in an extern block). *)
      (* `pub` controls visibility. `unsafe` may be used for functions with unsafe bodies. `async` for async functions. *)
      (* `extern AbiStringLiteral` here is used for defining the calling convention of a Ferra function, typically for exporting it. *)
      (* TODO: Generics, where-clauses *)

      ParameterList ::= "(" (Parameter ("," Parameter)*)? ")"
      Parameter       ::= AttributeListOpt IDENTIFIER (":" Type)?
      (* Example: fn process_data(#[ai.assume(nll="noalias")] data_slice: &mut [u8]) { ... } *)
      (* Example with optional types: fn calc(a, b) { ... } - types inferred *)
      (* Example with explicit types: fn calc(a: int, b: int) { ... } *)
      ```

  2.3 Data Class Declaration
      ```ebnf
      DataClassDecl ::= AttributeListOpt "data" IDENTIFIER "{" FieldList? "}"
      (* Example: data User { id: Int, name: String, email: String } *)
      (* Example: #[derive(Debug)] data Point { x: Int, y: Int } *)

      FieldList     ::= Field ("," Field)* (",")?
      Field         ::= AttributeListOpt IDENTIFIER ":" Type
      (* Example: data Config { #[serde.rename("max_items")] max_items_limit: Int } *) 
      ```

  2.4 External Block Declaration (New for FFI)
      ```ebnf
      ExternBlock ::= AttributeListOpt "extern" AbiStringLiteral "{" (ExternalItem)* "}"
      AbiStringLiteral ::= StringLiteral (* Lexically a string literal. Semantically, for FFI, common values include "C", "system". The compiler will validate allowed ABI strings. *)

      ExternalItem ::= ExternFunctionDecl
                     | ExternVariableDecl
                     (* | ExternTypeAliasDecl (* For declaring external C typedefs if needed *) *)

      ExternFunctionDecl ::= AttributeListOpt ("async")? "fn" IDENTIFIER ParameterList ( "->" Type )? ";"
      (* Note: Semicolon terminator, no Block body. `async` here is likely not applicable for standard C FFI. *)

      ExternVariableDecl ::= AttributeListOpt "static" IDENTIFIER ":" Type ";"
      (* Declares an external C global variable. Assumed immutable from Ferra's side (const). *)
      ```

---

## 3. Types

  (*
    This section defines the syntax for type expressions in Ferra.
    The goal for v0.1 is to support common and intuitive type forms like tuples,
    dynamic arrays/lists, function types, and generic types. More advanced forms
    like fixed-size arrays (e.g., `[T; N]`) and explicit pointer/reference types
    (e.g., `*T`, `&T`) are deferred for future consideration via RFCs, to keep the
    initial type grammar minimal and focused.
    FFI introduces raw pointer types and extern function types.
  *)
  ```ebnf
  Type ::= TupleType
         | ArrayType
         | FunctionType          (* Ferra's own function types *)
         | ExternFunctionType    (* For C-ABI function pointer types *)
         | RawPointerType        (* For *const T and *mut T *)
         | GenericType
         | QualifiedIdentifier
         | IDENTIFIER                (* Simple type name, e.g. Int, Float, String, User, c_int *)

  TupleType       ::= "(" ( Type ("," Type)* (",")? )? ")"
                  (*  Parser Hint: `()` is the unit type.
                      `(T)` is treated as a parenthesized `Type T`, not a single-element tuple.
                      A tuple type requires at least one comma, e.g., `(T,)` or `(T1, T2)`.
                      A trailing comma is allowed, e.g. `(T1, T2,)`. *)

  ArrayType       ::= "[" Type "]"
                  (* Represents a dynamic array/list/vector in v0.1, e.g., `[Int]`. *)

  FunctionType    ::= ("async")? "fn" "(" (ParameterTypeList)? ")" "->" Type
  ParameterTypeList ::= Type ("," Type)* (",")?
                  (* Represents the type of a function, e.g., `fn(String, Int) -> Bool`. *)

  GenericType     ::= IDENTIFIER "<" TypeArgumentList ">"
                  (* Represents a generic type instantiation, e.g., `Result<Response>`, `List<[Int]>`. *)
  TypeArgumentList ::= Type ("," Type)* (",")?

  QualifiedIdentifier ::= IDENTIFIER ("::" IDENTIFIER)+
                  (* Represents a namespaced type, e.g., `http::Client`. *)

  RawPointerType ::= "*" ("const" Type | "mut" Type)
                   (* Examples: *const i32, *mut User, *const c_void *)
                   (* `Type` here refers to the pointee type. *)
  
  ExternFunctionType ::= "extern" AbiStringLiteral "fn" "(" (ParameterTypeList)? ")" ("->" Type)?
                       (* Example: type MyCFnPtr = extern "C" fn(i32) -> i32; *)
                       (* AbiStringLiteral here also typically "C". *)
  ```

---

## 4. Expressions (Pratt-style precedence table)
  (*
    Ferra will use a Pratt parser (top-down operator-precedence parser) for expressions.
    This allows for intuitive handling of operator precedence and associativity.
    The actual EBNF rules for binary and unary operations are kept abstract here,
    as the Pratt parser's logic, guided by a precedence table (see Appendix A),
    will determine how these are constructed.
  *)
  ```ebnf
  Expression ::= Literal
               | IDENTIFIER
               | QualifiedIdentifier
               | FunctionCall
               | MethodCall
               | MatchExpr
               | AwaitExpr       (* Postfix .await *)
               | PostfixOpExpr   (* e.g., error propagation `?` *)
               | UnaryOpExpr
               | BinaryOpExpr
               | GroupedExpr
               | IfExpression    (* `if` can be an expression *)
               (* | ArrayLiteral, TupleLiteral, MapLiteral ... *)
               (* | LambdaExpr ... *)
               (* | ... other expression forms ... *)

  FunctionCall ::= IDENTIFIER ArgumentList
                 | QualifiedIdentifier ArgumentList
  MethodCall   ::= Expression "." IDENTIFIER ArgumentList
  ArgumentList ::= "(" (Expression ("," Expression)*)? ")"
  AwaitExpr    ::= Expression "." "await"
  GroupedExpr  ::= "(" Expression ")"

  UnaryOpExpr  ::= UNARY_OPERATOR Expression
  BinaryOpExpr ::= Expression BINARY_OPERATOR Expression
  PostfixOpExpr::= Expression POSTFIX_OPERATOR

  UNARY_OPERATOR ::= "!" | "-" | "+"
  BINARY_OPERATOR::= "+" | "-" | "*" | "/" | "%"  (* Arithmetic *)
                   | "==" | "!=" | "<" | "<=" | ">" | ">=" (* Comparison *)
                   | "&&" | "||" (* Logical AND, OR *)
                   | "&" | "|" | "^" | "<<" | ">>" (* Bitwise *)
                   | "??"           (* Nil-coalescing *)
                   | ".." | "..="    (* Range *)
                   (* Assignment operators are typically handled as statements or specific expression forms *)
  POSTFIX_OPERATOR ::= "?"

  IfExpression ::= "if" Expression Block "else" Block
                 (* `else` branch is mandatory for `if` in an expression context.
                    The parser might use the same AST node for IfStatement and IfExpression,
                    with a flag indicating its context or by type-checking requirements. *)
  ```
  (*
    Note on Assignments: While assignment operators (`=`, `+=`, etc.) are listed
    in the precedence table for completeness and to show their low precedence and
    right-associativity, assignment itself is often treated as a statement or a special
    expression form rather than a generic BinaryOpExpr in many languages to control side effects
    and return values (e.g., if assignments return a value or not).
    This will be clarified in the "Statements" section.
  *)

  2.3 Match Expression (can also be a statement, context-dependent)
      ```ebnf
      MatchExpr ::= "match" Expression "{" (MatchArm)+ "}"
      MatchArm  ::= Pattern "=>" Expression (";" | ",")? (* Terminator might depend on block structure *)
      (* Example: match u { User { name, .. } => "Hello, " + name } *)

      Pattern   ::= DataClassPattern
                  | Literal
                  | IDENTIFIER (* For binding *)
                  | "_" (* Wildcard *)
                  (* | ... other patterns ... *)

      DataClassPattern ::= IDENTIFIER "{" (FieldPattern ("," FieldPattern)* (",")? )? (".."?)? "}"
      FieldPattern     ::= IDENTIFIER (":" Pattern)?
                         | IDENTIFIER (* Shorthand for IDENTIFIER : IDENTIFIER *)
      ```

---

## 5. Statements
  (*
    Statements are the primary units of execution. Ferra uses a newline-sensitive termination
    approach, augmented by optional semicolons and context-aware rules for block endings.
    The conceptual `StatementTerminator` below is resolved by the parser based on actual
    semicolon tokens or the newline termination rules described in Section 1.1.
  *)
  ```ebnf
  Statement ::= LetDeclarationStatement
              | ExpressionStatement
              | BlockStatement
              | ReturnStatement
              | IfStatement
              | WhileStatement
              | ForStatement
              | BreakStatement
              | ContinueStatement
            (* | ... other statements ... *)

  (* Conceptual terminator - parser logic, not a distinct token *)
  (* StatementTerminator_Rule ::=
       SemicolonExists
     | (NewlineExists AND PrecedingTokenAllowsTermination AND NextTokenAllowsStatementStart)
     | IsFollowedByClosingBrace
     | IsEOF
  *)

  SemicolonOpt ::= (";")?

  LetDeclarationStatement ::= VariableDecl SemicolonOpt
  ExpressionStatement     ::= Expression SemicolonOpt
  BlockStatement          ::= Block

  ReturnStatement   ::= "return" (Expression)? SemicolonOpt
  BreakStatement    ::= "break" SemicolonOpt
  ContinueStatement ::= "continue" SemicolonOpt

  WhileStatement ::= "while" Expression Block
                 (* No SemicolonOpt needed here; Block ends the statement *)

  ForStatement   ::= "for" IDENTIFIER "in" Expression Block
                 (* No SemicolonOpt needed here; Block ends the statement *)

  IfStatement ::= "if" Expression Block ("else" (Block | IfStatement))?
              (* No SemicolonOpt needed here; Block or sub-IfStatement ends it *)

  Block          ::= BraceBlock | IndentedBlock

  BraceBlock    ::= "{" (Statement)* "}"
                (* The last statement inside a BraceBlock does not strictly need a semicolon
                   or a trailing newline before the `}`. *)

  IndentedBlock ::= INDENT (Statement)+ DEDENT
                (* INDENT and DEDENT are special tokens emitted by the lexer.
                   An IndentedBlock must contain at least one statement.
                   Statements within an IndentedBlock follow normal termination rules (newline or semicolon). *)
  ```

  (* Example demonstrating some control flow statements and block styles: *)
  ```ferra
  fn explicit_brace_sum(n: Int) -> Int {
      let mut total = 0
      for i in 0..n { // BraceBlock for `for` body
          if i % 2 == 0 { // BraceBlock for `if` body
              total = total + i
          }
      }
      return total
  }

  fn indented_sum(n: Int) -> Int:
      INDENT (* Start of IndentedBlock for function body *)
      let mut total = 0
      for i in 0..n:
          INDENT (* Start of IndentedBlock for `for` body *)
          if i % 2 == 0:
              INDENT (* Start of IndentedBlock for `if` body *)
              total = total + i
              DEDENT (* End of IndentedBlock for `if` body *)
          DEDENT (* End of IndentedBlock for `for` body *)
      return total
      DEDENT (* End of IndentedBlock for function body *)
  (* Note: The INDENT/DEDENT tokens in the example above are conceptual to show structure;
     they are not written by the Ferra programmer. The programmer uses actual indentation. *)

  let result = if explicit_brace_sum(10) > 20 { "big" } else { "small" }
  ```

  (* Note on Data-Parallel Loops:
     For v0.1, data-parallel iteration constructs (e.g., a parallel `for_each`) are planned
     to be exposed primarily through method calls on parallel iterator types obtained from
     collections (e.g., `my_vector.par_iter().for_each(...)`). This approach leverages
     Ferra's existing expression and method call grammar (Section 4) rather than introducing
     new dedicated loop keywords or statement syntax at this stage. Refer to the
     `DATA_PARALLEL_GPU.md` document (Section 2.2) for further details on the design
     of these data-parallel constructs.
  *)

---

## 6. Module & Macro Forms
  (* TODO *)
  ```ebnf
  (* ImportDeclaration ::= "import" ... *)
  MacroInvocation   ::= IDENTIFIER "!" (* Actual syntax TBD, e.g., parentheses, braces, specific delimiter *)
                      (* Example: json! { ... } -- The content of { ... } depends on the macro definition *) 
  ```

---

## Appendix A. Operator Precedence Table (Informative for Pratt Parser)

  This table guides the Pratt parser. Operators with higher precedence levels bind more tightly.

  | Level | Associativity | Operator Category      | Operators                                                        | Ferra Example (Conceptual)                |
  | :---- | :------------ | :--------------------- | :--------------------------------------------------------------- | :---------------------------------------- |
  | 15    | Postfix       | Error Propagation      | `?`                                                              | `might_fail()?`                           |
  | 14    | Left          | Member/Call/Index      | `.` (field/method access), `()` (function call) (*`[]` index TBD*) | `obj.field`, `func(x)`, `arr[0]`           |
  | 13    | Right         | Unary Prefix           | `!` (logical NOT), `-` (negation), `+` (unary plus)              | `!is_valid`, `-total`, `+offset`          |
  | 12    | Left          | Multiplicative         | `*`, `/`, `%`                                                    | `a * b`, `c / d`, `e % f`                 |
  | 11    | Left          | Additive               | `+`, `-`                                                         | `a + b`, `c - d`                          |
  | 10    | Left          | Bitwise Shift          | `<<`, `>>`                                                       | `bits << 2`, `value >> 1`                 |
  | 9     | Left          | Bitwise AND, XOR, OR   | `&` (AND), `^` (XOR), `                                          | ` (OR)` (Bitwise) | `x & mask`, `y ^ key`, `z              | flags` | 
  | 8     | N/A           | Range                  | `..` (exclusive), `..=` (inclusive)                              | `0..10`, `start..=end`                    |
  | 7     | None          | Relational/Comparison  | `==`, `!=`, `<`, `<=`, `>`, `>=`                                    | `a == b`, `c < d`                         |
  | 6     | Left          | Logical AND            | `&&` (primary), `and` (lexer alias)                              | `cond1 && cond2`, `is_ok and has_data`    |
  | 5     | Left          | Logical OR             | `                                                                |                                           | ` (primary), `or` (lexer alias)                                  | `opt1 `                                   | ` opt2`, `err1 or err2`                 |
  | 4     | Right         | Nil-Coalescing         | `??`                                                             | `optional_value ?? default`               |
  | 3     | N/A           | (Reserved for Pipeline)| (* `                                                            | >` - Not included in v0.1 *)             |                                           |
  | 2     | Right         | Assignment             | `