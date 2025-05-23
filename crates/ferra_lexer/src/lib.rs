// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Ferra Language Project Contributors

use std::iter::Peekable;
use std::str::CharIndices;

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
    ByteLiteral, // e.g. b'a', b"foo"

    // Comments (skipped by parser but useful for tooling)
    LineComment,  // `// ...`
    BlockComment, // `/* ... */`

    // Operators & Punctuation
    Plus,            // +
    Minus,           // -
    Star,            // *
    Slash,           // /
    Percent,         // %
    EqualEqual,      // ==
    NotEqual,        // !=
    Less,            // <
    Greater,         // >
    LessEqual,       // <=
    GreaterEqual,    // >=
    LogicalAnd,      // &&
    LogicalOr,       // ||
    BitAnd,          // &
    BitOr,           // |
    Caret,           // ^
    ShiftLeft,       // <<
    ShiftRight,      // >>
    Coalesce,        // ??
    Equal,           // =
    PlusEqual,       // +=
    MinusEqual,      // -=
    StarEqual,       // *=
    SlashEqual,      // /=
    PercentEqual,    // %=
    BitAndEqual,     // &=
    BitOrEqual,      // |=
    CaretEqual,      // ^=
    ShiftLeftEqual,  // <<=
    ShiftRightEqual, // >>=
    Bang,            // !
    Question,        // ?
    Dot,             // .
    Comma,           // ,
    Colon,           // :
    Semicolon,       // ;
    LParen,          // (
    RParen,          // )
    LBrace,          // {
    RBrace,          // }
    LBracket,        // [
    RBracket,        // ]
    Arrow,           // ->
    FatArrow,        // =>
    DotDot,          // ..
    DotDotEqual,     // ..=
    PathSep,         // ::
    Underscore,      // _

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
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Span {
    /// Dummy span for early‐stage tests; replace with real tracking later.
    pub fn dummy() -> Self {
        Span {
            start: Position {
                line: 0,
                column: 0,
                offset: 0,
            },
            end: Position {
                line: 0,
                column: 0,
                offset: 0,
            },
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

pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.char_indices().peekable(),
            line: 1,
            column: 1,
        }
    }

    pub fn lex(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(&(idx, ch)) = self.chars.peek() {
            if ch.is_ascii_digit()
                || (ch == '.'
                    && self
                        .peek_nth_char(1)
                        .is_some_and(|(_, c)| c.is_ascii_digit()))
            {
                tokens.push(self.lex_number());
                continue;
            }
            // Skip whitespace
            if ch.is_whitespace() {
                self.advance_char();
                continue;
            }

            // Identifiers and keywords: [a-zA-Z_][a-zA-Z0-9_]*
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = idx;
                let start_col = self.column;
                let mut end = idx;
                let mut end_col = self.column;
                let mut ident = String::new();
                while let Some(&(_, c)) = self.chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        ident.push(c);
                        let (j, _) = self.advance_char().unwrap();
                        end = j;
                        end_col = self.column;
                    } else {
                        break;
                    }
                }
                let kind = match ident.as_str() {
                    "let" => TokenKind::Let,
                    "var" => TokenKind::Var,
                    "fn" => TokenKind::Fn,
                    "async" => TokenKind::Async,
                    "data" => TokenKind::Data,
                    "match" => TokenKind::Match,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    _ => TokenKind::Identifier,
                };
                tokens.push(Token {
                    kind,
                    lexeme: ident,
                    literal: None,
                    span: Span {
                        start: Position {
                            line: self.line,
                            column: start_col,
                            offset: start,
                        },
                        end: Position {
                            line: self.line,
                            column: end_col,
                            offset: end,
                        },
                    },
                });
                continue;
            }

            // Multi-character operators and punctuation (maximal munch)
            let multi_char = [
                ("==", TokenKind::EqualEqual),
                ("!=", TokenKind::NotEqual),
                ("<=", TokenKind::LessEqual),
                (">=", TokenKind::GreaterEqual),
                ("&&", TokenKind::LogicalAnd),
                ("||", TokenKind::LogicalOr),
                ("+=", TokenKind::PlusEqual),
                ("-=", TokenKind::MinusEqual),
                ("*=", TokenKind::StarEqual),
                ("/=", TokenKind::SlashEqual),
                ("%=", TokenKind::PercentEqual),
                ("&=", TokenKind::BitAndEqual),
                ("|=", TokenKind::BitOrEqual),
                ("^=", TokenKind::CaretEqual),
                ("<<=", TokenKind::ShiftLeftEqual),
                ("<<", TokenKind::ShiftLeft),
                (">>=", TokenKind::ShiftRightEqual),
                (">>", TokenKind::ShiftRight),
                ("->", TokenKind::Arrow),
                ("=>", TokenKind::FatArrow),
                ("..=", TokenKind::DotDotEqual),
                ("..", TokenKind::DotDot),
                ("::", TokenKind::PathSep),
                ("??", TokenKind::Coalesce),
            ];
            let mut matched = false;
            for (op, kind) in multi_char.iter() {
                if self.input[idx..].starts_with(op) {
                    tokens.push(Token {
                        kind: kind.clone(),
                        lexeme: op.to_string(),
                        literal: None,
                        span: Span {
                            start: Position {
                                line: self.line,
                                column: self.column,
                                offset: idx,
                            },
                            end: Position {
                                line: self.line,
                                column: self.column + op.len(),
                                offset: idx + op.len(),
                            },
                        },
                    });
                    for _ in 0..op.len() {
                        self.advance_char();
                    }
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }

            // Single-char punctuation
            let kind = match ch {
                '=' => TokenKind::Equal,
                ';' => TokenKind::Semicolon,
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                ',' => TokenKind::Comma,
                ':' => TokenKind::Colon,
                '+' => TokenKind::Plus,
                '-' => TokenKind::Minus,
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                _ => TokenKind::Error,
            };
            let lexeme = ch.to_string();
            tokens.push(Token {
                kind,
                lexeme: lexeme.clone(),
                literal: None,
                span: Span {
                    start: Position {
                        line: self.line,
                        column: self.column,
                        offset: idx,
                    },
                    end: Position {
                        line: self.line,
                        column: self.column + 1,
                        offset: idx + ch.len_utf8(),
                    },
                },
            });
            self.advance_char();
        }
        tokens.push(Token::eof_dummy());
        tokens
    }

    fn advance_char(&mut self) -> Option<(usize, char)> {
        let next = self.chars.next();
        if let Some((_, ch)) = next {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        next
    }

    fn peek_nth_char(&mut self, n: usize) -> Option<(usize, char)> {
        self.chars.clone().nth(n)
    }

    fn current_offset(&mut self) -> usize {
        self.chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.input.len())
    }

    /// Scan an integer or float, honoring underscores and exponents.
    fn lex_number(&mut self) -> Token {
        let start_offset = self.chars.peek().unwrap().0;
        let start_line = self.line;
        let start_col = self.column;

        let mut lexeme = String::new();
        let mut has_dot = false;
        let mut has_exp = false;

        // Fix: allow leading dot (e.g. .5)
        if let Some(&(_, ch)) = self.chars.peek() {
            if ch == '.' {
                has_dot = true;
                lexeme.push('.');
                self.advance_char();
            }
        }

        // Integer part (or fraction if leading dot)
        while let Some(&(_, ch)) = self.chars.peek() {
            match ch {
                '0'..='9' => {
                    lexeme.push(ch);
                    self.advance_char();
                }
                '_' => {
                    lexeme.push(ch);
                    self.advance_char();
                }
                // Fractional part: allow dot if not already seen and not in exponent
                '.' if !has_dot && !has_exp => {
                    has_dot = true;
                    lexeme.push('.');
                    self.advance_char();
                }
                // Exponent part: only once
                'e' | 'E' if !has_exp => {
                    has_exp = true;
                    has_dot = true;
                    lexeme.push(ch);
                    self.advance_char();
                    // Optional + or –
                    if let Some((_, next_ch)) = self.chars.peek() {
                        if *next_ch == '+' || *next_ch == '-' {
                            lexeme.push(*next_ch);
                            self.advance_char();
                        }
                    }
                }
                _ => break,
            }
        }

        // Fix: allow trailing dot (e.g. 7.) as float
        if !has_dot {
            // If the lexeme ends with a dot, treat as float
            if lexeme.ends_with('.') {
                has_dot = true;
            }
        }

        let end_offset = self.current_offset();
        let span = Span {
            start: Position {
                line: start_line,
                column: start_col,
                offset: start_offset,
            },
            end: Position {
                line: self.line,
                column: self.column,
                offset: end_offset,
            },
        };

        // Strip underscores for parsing
        let cleaned: String = lexeme.chars().filter(|&c| c != '_').collect();

        // Decide kind and parse value
        if has_dot {
            match cleaned.parse::<f64>() {
                Ok(val) => Token {
                    kind: TokenKind::FloatLiteral,
                    lexeme,
                    literal: Some(LiteralValue::Float(val)),
                    span,
                },
                Err(_) => Token {
                    kind: TokenKind::Error,
                    lexeme,
                    literal: None,
                    span,
                },
            }
        } else {
            match cleaned.parse::<i64>() {
                Ok(val) => Token {
                    kind: TokenKind::IntegerLiteral,
                    lexeme,
                    literal: Some(LiteralValue::Integer(val)),
                    span,
                },
                Err(_) => Token {
                    kind: TokenKind::Error,
                    lexeme,
                    literal: None,
                    span,
                },
            }
        }
    }
}
