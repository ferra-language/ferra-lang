//! Token type definitions and related utilities
//!
//! These types will eventually interface with the lexer output.
//! For now, they provide a mock interface for development.

/// Source location information for tokens
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    pub fn dummy() -> Self {
        Self::new(0, 0, 1, 1)
    }

    /// Combine two spans into one that covers both
    pub fn combine(&self, other: Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,     // Use the start line
            column: self.column, // Use the start column
        }
    }
}

/// Token types as they would come from the lexer
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    BooleanLiteral(bool),

    // Identifiers
    Identifier(String),

    // Keywords
    Let,
    Var,
    Fn,
    Async,
    Data,
    Match,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Break,
    Continue,
    Extern,
    Static,
    Pub,
    Unsafe,
    Where, // for generic where clauses

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AmpAmp,   // && (also `and` keyword)
    PipePipe, // || (also `or` keyword)
    Bang,
    Question,
    QuestionQuestion, // ??
    Pipe,             // | (for OR patterns in match expressions)

    // Assignment operators
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    DotDot,      // .. (for range patterns and slice patterns)
    DotDotEqual, // ..= (for inclusive range patterns)
    Semicolon,
    Colon,
    DoubleColon, // :: (for path separators like std::vec::Vec)
    Arrow,       // ->
    FatArrow,    // => (for match expressions and macro rules)
    Hash,        // # (for attributes like #[derive(Debug)])
    At,          // @ (for alternative attribute syntax like @inline and binding patterns)
    Apostrophe,  // ' (for lifetimes like 'a, 'static)
    Ampersand,   // & (for references)

    // Special tokens for indentation
    Indent,
    Dedent,
    Newline,

    // End of file
    Eof,
}

/// A token with its type and location information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, span: Span) -> Self {
        Self { token_type, span }
    }

    pub fn dummy(token_type: TokenType) -> Self {
        Self::new(token_type, Span::dummy())
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.token_type, TokenType::Eof)
    }
}
