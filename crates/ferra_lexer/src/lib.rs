// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Ferra Language Project Contributors

/// All the different token kinds the Ferra lexer can emit.
/// Marked non_exhaustive so we can add new variants (raw strings, etc.) later.
#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Keywords
    Let,
    Var,
    Fn,
    Async,
    Data,
    Match,
    True,
    False,

    // Identifiers
    Identifier,

    // Literals
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    CharacterLiteral,
    BooleanLiteral,
    ByteLiteral,        // e.g. b'a', b"foo"

    // Comments (skipped by parser but useful for tooling)
    LineComment,       // `// ...`
    BlockComment,      // `/* ... */`

    // Operators & Punctuation
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    EqualEqual,     // ==
    NotEqual,       // !=
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    LogicalAnd,     // &&
    LogicalOr,      // ||
    BitAnd,         // &
    BitOr,          // |
    Caret,          // ^
    ShiftLeft,      // <<
    ShiftRight,     // >>
    Coalesce,       // ??
    Equal,          // =
    PlusEqual,      // +=
    MinusEqual,     // -=
    StarEqual,      // *=
    SlashEqual,     // /=
    PercentEqual,   // %=
    BitAndEqual,    // &=
    BitOrEqual,     // |=
    CaretEqual,     // ^=
    ShiftLeftEqual, // <<=
    ShiftRightEqual,// >>=    
    Bang,           // !
    Question,       // ?
    Dot,            // .
    Comma,          // ,
    Colon,          // :
    Semicolon,      // ;
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Arrow,          // ->
    FatArrow,       // =>
    DotDot,         // ..
    DotDotEqual,    // ..=
    PathSep,        // ::
    Underscore,     // _

    // Structural tokens
    Indent,
    Dedent,
    Newline,
    Eof,

    // Fallback for unrecognized input
    Error,
}

/// A fully‐fledged token with metadata (kind, lexeme, literal, span).
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    /// The exact slice of source text.
    pub lexeme: String,
    /// Parsed literal value, if any.
    pub literal: Option<LiteralValue>,
    /// Source‐location span for diagnostics.
    pub span: Span,
}

/// Different literal values a token might carry.
#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),
    Byte(u8),
}

/// Precise span (start/end positions) in the source file.
#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

/// A single position: line, column, and byte offset.
#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub line:   usize,
    pub column: usize,
    pub offset: usize,
}

impl Span {
    /// Dummy span for early‐stage tests; replace with real tracking later.
    pub fn dummy() -> Self {
        Span {
            start: Position { line: 0, column: 0, offset: 0 },
            end:   Position { line: 0, column: 0, offset: 0 },
        }
    }
}

impl Token {
    /// Helper to emit a bare EOF token in tests.
    pub fn eof_dummy() -> Self {
        Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            literal: None,
            span: Span::dummy(),
        }
    }
}

/// Stub lexer: for now it only emits an EOF token.
/// We'll flesh this out to recognize keywords, identifiers, and all other tokens next. 
pub fn lex(_source: &str) -> Vec<Token> {
    vec![Token::eof_dummy()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_emits_eof() {
        let tokens = lex("");
        assert_eq!(tokens, vec![Token::eof_dummy()]);
    }
}
