// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Ferra Language Project Contributors

use std::iter::Peekable;
use std::str::CharIndices;
use unicode_ident::{is_xid_continue, is_xid_start};
use unicode_normalization::UnicodeNormalization;

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
    // LineComment,  // `// ...`
    // BlockComment, // `/* ... */`

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
    indent_stack: Vec<usize>, // track indentation levels
    pending_dedents: usize,   // track dedents to emit
    at_line_start: bool,      // are we at the start of a new line?
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.char_indices().peekable(),
            line: 1,
            column: 1,
            indent_stack: vec![0],
            pending_dedents: 0,
            at_line_start: true,
        }
    }

    pub fn lex(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(&(idx, ch)) = self.chars.peek() {
            // Handle dedents first
            if self.pending_dedents > 0 {
                self.pending_dedents -= 1;
                tokens.push(Token {
                    kind: TokenKind::Dedent,
                    lexeme: String::new(),
                    literal: None,
                    span: Span {
                        start: Position {
                            line: self.line,
                            column: self.column,
                            offset: idx,
                        },
                        end: Position {
                            line: self.line,
                            column: self.column,
                            offset: idx,
                        },
                    },
                });
                continue;
            }
            // Indentation logic at start of line
            if self.at_line_start {
                let mut col = 0;
                let mut off = idx;
                let mut chars_clone = self.chars.clone();
                while let Some(&(i, c)) = chars_clone.peek() {
                    if c == ' ' {
                        col += 1;
                        off = i + 1;
                        chars_clone.next();
                    } else if c == '\t' {
                        col += 4; // treat tab as 4 spaces
                        off = i + 1;
                        chars_clone.next();
                    } else {
                        break;
                    }
                }
                if let Some(&(_, c)) = chars_clone.peek() {
                    if c == '\n' {
                        // blank line, just emit Newline
                        self.chars = chars_clone;
                        self.at_line_start = true;
                        let start_offset = idx;
                        let start_col = self.column;
                        self.advance_char();
                        tokens.push(Token {
                            kind: TokenKind::Newline,
                            lexeme: "\n".to_string(),
                            literal: None,
                            span: Span {
                                start: Position {
                                    line: self.line - 1,
                                    column: start_col,
                                    offset: start_offset,
                                },
                                end: Position {
                                    line: self.line - 1,
                                    column: start_col + 1,
                                    offset: start_offset + 1,
                                },
                            },
                        });
                        continue;
                    }
                }
                let current_indent = *self.indent_stack.last().unwrap();
                if col > current_indent {
                    self.indent_stack.push(col);
                    tokens.push(Token {
                        kind: TokenKind::Indent,
                        lexeme: String::new(),
                        literal: None,
                        span: Span {
                            start: Position {
                                line: self.line,
                                column: 1,
                                offset: idx,
                            },
                            end: Position {
                                line: self.line,
                                column: col + 1,
                                offset: off,
                            },
                        },
                    });
                } else if col < current_indent {
                    while col < *self.indent_stack.last().unwrap() {
                        self.indent_stack.pop();
                        self.pending_dedents += 1;
                    }
                    continue;
                }
                // Advance chars to after leading whitespace
                while self
                    .chars
                    .peek()
                    .is_some_and(|&(_, c)| c == ' ' || c == '\t')
                {
                    self.advance_char();
                }
                self.at_line_start = false;
            }
            // Handle Newlines
            if ch == '\n' {
                let start_offset = idx;
                let start_col = self.column;
                self.advance_char();
                tokens.push(Token {
                    kind: TokenKind::Newline,
                    lexeme: "\n".to_string(),
                    literal: None,
                    span: Span {
                        start: Position {
                            line: self.line - 1,
                            column: start_col,
                            offset: start_offset,
                        },
                        end: Position {
                            line: self.line - 1,
                            column: start_col + 1,
                            offset: start_offset + 1,
                        },
                    },
                });
                self.at_line_start = true;
                continue;
            }
            if ch.is_ascii_digit()
                || (ch == '.'
                    && self
                        .peek_nth_char(1)
                        .is_some_and(|(_, c)| c.is_ascii_digit()))
            {
                tokens.push(self.lex_number());
                continue;
            }

            // Comments
            if ch == '/' {
                if self.peek_nth_char(1).is_some_and(|(_, c)| c == '/') {
                    // Line comment
                    self.advance_char(); // consume '/'
                    self.advance_char(); // consume '/'
                    while let Some(&(_i, c)) = self.chars.peek() {
                        if c == '\n' {
                            break; // End of line comment
                        }
                        self.advance_char();
                    }
                    continue;
                } else if self.peek_nth_char(1).is_some_and(|(_, c)| c == '*') {
                    // Block comment (with nesting support)
                    let comment_start_line = self.line;
                    let comment_start_col = self.column;
                    let comment_start_offset = idx;

                    self.advance_char(); // consume '/'
                    self.advance_char(); // consume '*

                    let mut nesting_level = 1;
                    let mut closed = false;

                    while let Some(&(_i, c1)) = self.chars.peek() {
                        self.advance_char(); // Consume current char
                        if c1 == '/' {
                            if let Some(&(_j, c2)) = self.chars.peek() {
                                if c2 == '*' {
                                    // Start of a nested block comment
                                    self.advance_char(); // Consume '*
                                    nesting_level += 1;
                                }
                            }
                        } else if c1 == '*' {
                            if let Some(&(_j, c2)) = self.chars.peek() {
                                if c2 == '/' {
                                    // End of a block comment
                                    self.advance_char(); // Consume '/'
                                    nesting_level -= 1;
                                    if nesting_level == 0 {
                                        closed = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !closed {
                        tokens.push(Token {
                            kind: TokenKind::Error,
                            lexeme: self.input.get(comment_start_offset..self.current_offset()).unwrap_or("").to_string(),
                            literal: Some(LiteralValue::String(
                                "Unterminated block comment: expected closing */ before end of file.".to_string(),
                            )),
                            span: Span {
                                start: Position {
                                    line: comment_start_line,
                                    column: comment_start_col,
                                    offset: comment_start_offset,
                                },
                                end: Position {
                                    line: self.line,     // Current line at EOF or error point
                                    column: self.column, // Current column at EOF or error point
                                    offset: self.current_offset(),
                                },
                            },
                        });
                    }
                    continue;
                }
            }

            // Skip other whitespace
            if ch.is_whitespace() {
                // but not newline, handled above
                self.advance_char();
                continue;
            }

            // String Literals: "..."
            if ch == '"' {
                tokens.push(self.lex_string_literal());
                continue;
            }

            // Character Literals: '...'
            if ch == '\'' {
                tokens.push(self.lex_char_literal());
                continue;
            }

            // Underscore as punctuation if not part of an identifier
            if ch == '_' {
                // Peek next char: if it's not alphanumeric or _, treat as Underscore
                if let Some(&(_, next_ch)) = self.peek_nth_char(1).as_ref() {
                    if !next_ch.is_ascii_alphanumeric() && next_ch != '_' {
                        let start_offset = idx;
                        let start_col = self.column;
                        self.advance_char();
                        let end_offset = self.current_offset();
                        tokens.push(Token {
                            kind: TokenKind::Underscore,
                            lexeme: "_".to_string(),
                            literal: None,
                            span: Span {
                                start: Position {
                                    line: self.line,
                                    column: start_col,
                                    offset: start_offset,
                                },
                                end: Position {
                                    line: self.line,
                                    column: self.column,
                                    offset: end_offset,
                                },
                            },
                        });
                        continue;
                    }
                } else {
                    // End of input, so treat as Underscore
                    let start_offset = idx;
                    let start_col = self.column;
                    self.advance_char();
                    let end_offset = self.current_offset();
                    tokens.push(Token {
                        kind: TokenKind::Underscore,
                        lexeme: "_".to_string(),
                        literal: None,
                        span: Span {
                            start: Position {
                                line: self.line,
                                column: start_col,
                                offset: start_offset,
                            },
                            end: Position {
                                line: self.line,
                                column: self.column,
                                offset: end_offset,
                            },
                        },
                    });
                    continue;
                }
            }

            // Byte literals: b'a' or b"foo"
            if ch == 'b' {
                if let Some(&(_, next_ch)) = self.peek_nth_char(1).as_ref() {
                    if next_ch == '\'' || next_ch == '"' {
                        let start_offset = idx;
                        let start_line = self.line;
                        let start_col = self.column;
                        self.advance_char(); // consume 'b'
                        let quote = self.advance_char().unwrap().1; // consume quote
                        let mut content = String::new();
                        let mut closed = false;
                        while let Some(&(_, c)) = self.chars.peek() {
                            if c == quote {
                                self.advance_char();
                                closed = true;
                                break;
                            } else if c == '\\' {
                                self.advance_char();
                                if let Some(&(_, esc)) = self.chars.peek() {
                                    content.push('\\');
                                    content.push(esc);
                                    self.advance_char();
                                } else {
                                    break;
                                }
                            } else {
                                content.push(c);
                                self.advance_char();
                            }
                        }
                        let end_offset = self.current_offset();
                        let lexeme = self.input.get(start_offset..end_offset).unwrap_or("");
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
                        if !closed {
                            tokens.push(Token {
                                kind: TokenKind::Error,
                                lexeme: lexeme.to_string(),
                                literal: Some(LiteralValue::String(
                                    "Unterminated byte literal".to_string(),
                                )),
                                span,
                            });
                        } else if quote == '\'' && content.len() != 1 {
                            tokens.push(Token {
                                kind: TokenKind::Error,
                                lexeme: lexeme.to_string(),
                                literal: Some(LiteralValue::String(
                                    "Byte literal must be exactly one byte".to_string(),
                                )),
                                span,
                            });
                        } else if quote == '\'' {
                            tokens.push(Token {
                                kind: TokenKind::ByteLiteral,
                                lexeme: lexeme.to_string(),
                                literal: Some(LiteralValue::Byte(content.as_bytes()[0])),
                                span,
                            });
                        } else {
                            tokens.push(Token {
                                kind: TokenKind::ByteLiteral,
                                lexeme: lexeme.to_string(),
                                literal: Some(LiteralValue::String(content)),
                                span,
                            });
                        }
                        continue;
                    }
                }
            }

            // Identifiers and keywords (Unicode-aware)
            if is_xid_start(ch) {
                // Use is_xid_start directly
                let start = idx;
                let start_col = self.column;
                let mut ident_str = String::new();
                ident_str.push(ch);
                self.advance_char();

                while let Some(&(_, c)) = self.chars.peek() {
                    if is_xid_continue(c) {
                        // Use is_xid_continue directly
                        ident_str.push(c);
                        self.advance_char();
                    } else {
                        break;
                    }
                }
                let end_offset = self.current_offset();

                // NFC Normalization
                let normalized_ident: String = ident_str.nfc().collect();

                let kind = match normalized_ident.as_str() {
                    "let" => TokenKind::Let,
                    "var" => TokenKind::Var,
                    "fn" => TokenKind::Fn,
                    "async" => TokenKind::Async,
                    "data" => TokenKind::Data,
                    "match" => TokenKind::Match,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "and" => TokenKind::LogicalAnd,
                    "or" => TokenKind::LogicalOr,
                    _ => TokenKind::Identifier,
                };

                let literal_value = match kind {
                    TokenKind::True => Some(LiteralValue::Boolean(true)),
                    TokenKind::False => Some(LiteralValue::Boolean(false)),
                    _ => None,
                };

                tokens.push(Token {
                    kind,
                    lexeme: normalized_ident.to_string(),
                    literal: literal_value,
                    span: Span {
                        start: Position {
                            line: self.line,
                            column: start_col,
                            offset: start,
                        },
                        end: Position {
                            line: self.line,
                            column: self.column,
                            offset: end_offset,
                        },
                    },
                });
                continue;
            }

            // Multi-character operators and punctuation (maximal munch)
            let multi_char = [
                ("<<=", TokenKind::ShiftLeftEqual),
                (">>=", TokenKind::ShiftRightEqual),
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
                ("<<", TokenKind::ShiftLeft),
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
                if self.input.get(idx..).unwrap_or("").starts_with(op) {
                    let start_offset = idx;
                    let start_col = self.column;
                    for _ in 0..op.len() {
                        self.advance_char();
                    }
                    let end_offset = self.current_offset();
                    tokens.push(Token {
                        kind: kind.clone(),
                        lexeme: op.to_string(),
                        literal: None,
                        span: Span {
                            start: Position {
                                line: self.line,
                                column: start_col,
                                offset: start_offset,
                            },
                            end: Position {
                                line: self.line,
                                column: self.column,
                                offset: end_offset,
                            },
                        },
                    });
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
                '<' => TokenKind::Less,
                '>' => TokenKind::Greater,
                '!' => TokenKind::Bang,
                '.' => TokenKind::Dot,
                '&' => TokenKind::BitAnd,
                '|' => TokenKind::BitOr,
                '^' => TokenKind::Caret,
                '_' => TokenKind::Underscore,
                '%' => TokenKind::Percent,
                '?' => TokenKind::Question,
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

            // At the very end, if nothing matched, emit Error for unrecognized input
            if !ch.is_ascii() || ch.is_control() {
                let start_offset = idx;
                let start_col = self.column;
                self.advance_char();
                let end_offset = self.current_offset();
                tokens.push(Token {
                    kind: TokenKind::Error,
                    lexeme: self
                        .input
                        .get(start_offset..end_offset)
                        .unwrap_or("")
                        .to_string(),
                    literal: Some(LiteralValue::String("Unrecognized input".to_string())),
                    span: Span {
                        start: Position {
                            line: self.line,
                            column: start_col,
                            offset: start_offset,
                        },
                        end: Position {
                            line: self.line,
                            column: self.column,
                            offset: end_offset,
                        },
                    },
                });
                continue;
            }
        }
        // At EOF, flush any remaining dedents
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token {
                kind: TokenKind::Dedent,
                lexeme: String::new(),
                literal: None,
                span: Span {
                    start: Position {
                        line: self.line,
                        column: self.column,
                        offset: self.current_offset(),
                    },
                    end: Position {
                        line: self.line,
                        column: self.column,
                        offset: self.current_offset(),
                    },
                },
            });
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
        let mut base = 10;

        // Check for 0x, 0o, 0b prefixes
        if self.chars.peek().is_some_and(|&(_, c)| c == '0') {
            lexeme.push('0');
            self.advance_char();
            if let Some(&(_, next_char)) = self.chars.peek() {
                match next_char.to_ascii_lowercase() {
                    'x' => {
                        base = 16;
                        lexeme.push(next_char);
                        self.advance_char();
                    }
                    'o' => {
                        base = 8;
                        lexeme.push(next_char);
                        self.advance_char();
                    }
                    'b' => {
                        base = 2;
                        lexeme.push(next_char);
                        self.advance_char();
                    }
                    '0'..='9' | '.' | 'e' | 'E' => { /* stay base 10 */ }
                    _ => {}
                }
            }
        }

        let mut has_dot = false;
        let mut has_exp = false;

        // Allow leading dot for base 10 floats (e.g., .5)
        if base == 10
            && self.chars.peek().is_some_and(|&(_, ch)| ch == '.')
            && self
                .peek_nth_char(1)
                .is_some_and(|(_, c)| c.is_ascii_digit())
        {
            has_dot = true;
            lexeme.push('.');
            self.advance_char();
        }

        // Digit loop
        while let Some(&(_, ch)) = self.chars.peek() {
            match ch {
                '0'..='9' if ch.is_digit(base) => {
                    lexeme.push(ch);
                    self.advance_char();
                }
                'a'..='f' | 'A'..='F' if base == 16 => {
                    lexeme.push(ch);
                    self.advance_char();
                }
                '_' => {
                    if lexeme
                        .ends_with(|c: char| c.is_digit(base) || (base != 10 && lexeme.len() == 2))
                    {
                        lexeme.push(ch);
                        self.advance_char();
                    } else {
                        break;
                    }
                }
                // Float specific for base 10
                '.' if base == 10 && !has_dot && !has_exp => {
                    // If next char is a digit, treat as float part
                    if self
                        .peek_nth_char(1)
                        .is_some_and(|(_, c)| c.is_ascii_digit())
                    {
                        has_dot = true;
                        lexeme.push('.');
                        self.advance_char();
                    } else {
                        // If next char is not a digit, treat as float with trailing dot (e.g., 7.)
                        has_dot = true;
                        lexeme.push('.');
                        self.advance_char();
                        break;
                    }
                }
                'e' | 'E' if base == 10 && !has_exp => {
                    has_exp = true;
                    has_dot = true;
                    lexeme.push(ch);
                    self.advance_char();
                    if let Some((_, next_ch)) = self.chars.peek() {
                        if *next_ch == '+' || *next_ch == '-' {
                            lexeme.push(*next_ch);
                            self.advance_char();
                        }
                    }
                    if !self.chars.peek().is_some_and(|&(_, c)| c.is_ascii_digit()) {
                        break;
                    }
                }
                _ => break,
            }
        }

        // Validate that if a prefix was consumed, there are actual digits after it
        if (base == 16 && lexeme.eq_ignore_ascii_case("0x"))
            || (base == 8 && lexeme.eq_ignore_ascii_case("0o"))
            || (base == 2 && lexeme.eq_ignore_ascii_case("0b"))
        {
            let end_offset = self.current_offset();
            return Token {
                kind: TokenKind::Error,
                lexeme: lexeme.clone(),
                literal: Some(LiteralValue::String(format!(
                    "Expected digits after base prefix '{}', but found none",
                    &lexeme
                ))),
                span: Span {
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
                },
            };
        }
        if lexeme.ends_with('_') {
            let end_offset = self.current_offset();
            return Token {
                kind: TokenKind::Error,
                lexeme: lexeme.clone(),
                literal: Some(LiteralValue::String(format!("Number literal cannot end with an underscore: '{}'. Remove the trailing underscore.", lexeme))),
                span: Span {
                    start: Position { line: start_line, column: start_col, offset: start_offset },
                    end: Position { line: self.line, column: self.column, offset: end_offset },
                },
            };
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

        let cleaned_lexeme: String = lexeme.chars().filter(|&c| c != '_').collect();

        if has_dot {
            match cleaned_lexeme.parse::<f64>() {
                Ok(val) => Token {
                    kind: TokenKind::FloatLiteral,
                    lexeme: lexeme.clone(),
                    literal: Some(LiteralValue::Float(val)),
                    span,
                },
                Err(_) => Token {
                    kind: TokenKind::Error,
                    lexeme: lexeme.clone(),
                    literal: Some(LiteralValue::String(format!("Invalid float literal: '{}'. Expected a valid float (e.g., 1.23, 4e5, 7.), but got an invalid format.", lexeme))),
                    span,
                },
            }
        } else {
            let value_str_to_parse = if base != 10 {
                if cleaned_lexeme.starts_with("0x")
                    || cleaned_lexeme.starts_with("0X")
                    || cleaned_lexeme.starts_with("0o")
                    || cleaned_lexeme.starts_with("0O")
                    || cleaned_lexeme.starts_with("0b")
                    || cleaned_lexeme.starts_with("0B")
                {
                    &cleaned_lexeme[2..]
                } else {
                    cleaned_lexeme.as_str()
                }
            } else {
                cleaned_lexeme.as_str()
            };

            if value_str_to_parse.is_empty() && lexeme == "0" && base == 10 {
                return Token {
                    kind: TokenKind::IntegerLiteral,
                    lexeme: "0".to_string(),
                    literal: Some(LiteralValue::Integer(0)),
                    span,
                };
            }
            if value_str_to_parse.is_empty() && base != 10 {
                return Token {
                    kind: TokenKind::Error,
                    lexeme: lexeme.clone(),
                    literal: Some(LiteralValue::String(format!("Invalid integer literal for base {}: '{}'. Expected only valid digits for this base.", base, lexeme))),
                    span,
                };
            }

            match i64::from_str_radix(value_str_to_parse, base) {
                Ok(val) => Token {
                    kind: TokenKind::IntegerLiteral,
                    lexeme: lexeme.clone(),
                    literal: Some(LiteralValue::Integer(val)),
                    span,
                },
                Err(_) => Token {
                    kind: TokenKind::Error,
                    lexeme: lexeme.clone(),
                    literal: Some(LiteralValue::String(format!("Invalid integer literal for base {}: '{}'. Expected only valid digits for this base.", base, lexeme))),
                    span,
                },
            }
        }
    }

    fn lex_string_literal(&mut self) -> Token {
        let start_offset = self.current_offset();
        let start_line = self.line;
        let start_col = self.column;

        let mut content = String::new();

        self.advance_char(); // consume the opening quote

        let mut closed = false;
        while let Some(&(_idx, ch)) = self.chars.peek() {
            match ch {
                '\"' => {
                    self.advance_char(); // consume the closing quote
                    closed = true;
                    break;
                }
                '\\' => {
                    self.advance_char(); // consume backslash
                    if let Some(&(_escaped_idx, next_ch)) = self.chars.peek() {
                        match next_ch {
                            'n' => content.push('\n'),
                            't' => content.push('\t'),
                            '\\' => content.push('\\'),
                            '\"' => content.push('\"'),
                            // TODO: Add \u{...} handling here as per DESIGN_LEXER.md
                            _ => {
                                // Invalid escape: emit error token
                                let current_lex_end_offset = self.current_offset();
                                let current_lex_end_col = self.column;
                                let current_lex_end_line = self.line;
                                let lexeme = self
                                    .input
                                    .get(start_offset..current_lex_end_offset + 1)
                                    .unwrap_or("");
                                self.advance_char(); // consume the invalid escape char
                                return Token {
                                    kind: TokenKind::Error,
                                    lexeme: lexeme.to_string(),
                                    literal: Some(LiteralValue::String(format!("Invalid escape sequence in string literal: \\{}. Only valid escapes are \\n, \\t, \\\\, \\\" and Unicode escapes.", next_ch))),
                                    span: Span {
                                        start: Position {
                                            line: start_line,
                                            column: start_col,
                                            offset: start_offset,
                                        },
                                        end: Position {
                                            line: current_lex_end_line,
                                            column: current_lex_end_col + 1,
                                            offset: current_lex_end_offset + 1,
                                        },
                                    },
                                };
                            }
                        }
                        self.advance_char(); // consume the character after backslash (if not error)
                    } else {
                        break; // Unterminated string: EOF after backslash
                    }
                }
                '\n' => {
                    // An unescaped newline character.
                    // According to DESIGN_LEXER.md, string literals are single line unless escaped.
                    // This signifies an unterminated string.
                    // The newline character itself is NOT part of the string literal's lexeme or content.
                    break;
                }
                _ => {
                    content.push(ch);
                    self.advance_char();
                }
            }
        }

        // current_offset() reflects the position *after* the last consumed character.
        // If closed, it's after the closing quote. If not closed, it's where it stopped.
        let current_lex_end_offset = self.current_offset();
        let current_lex_end_col = self.column;
        let current_lex_end_line = self.line;

        if !closed {
            return Token {
                kind: TokenKind::Error,
                lexeme: self.input.get(start_offset..current_lex_end_offset).unwrap_or("").to_string(),
                literal: Some(LiteralValue::String(
                    "Unterminated string literal: expected closing quote \" before end of line or file.".to_string(),
                )),
                span: Span {
                    start: Position {
                        line: start_line,
                        column: start_col,
                        offset: start_offset,
                    },
                    end: Position {
                        line: current_lex_end_line,
                        column: current_lex_end_col,
                        offset: current_lex_end_offset,
                    },
                },
            };
        }

        Token {
            kind: TokenKind::StringLiteral,
            lexeme: self
                .input
                .get(start_offset..current_lex_end_offset)
                .unwrap_or("")
                .to_string(),
            literal: Some(LiteralValue::String(content)),
            span: Span {
                start: Position {
                    line: start_line,
                    column: start_col,
                    offset: start_offset,
                },
                end: Position {
                    line: current_lex_end_line, // if an escaped newline was processed, self.line would be updated by advance_char
                    column: current_lex_end_col,
                    offset: current_lex_end_offset,
                },
            },
        }
    }

    fn lex_char_literal(&mut self) -> Token {
        let start_offset = self.current_offset();
        let start_line = self.line;
        let start_col = self.column;

        self.advance_char(); // consume the opening quote '

        let mut char_val: Option<char> = None;
        let mut closed = false;
        let mut error_msg: Option<String> = None;
        let mut consumed_char_count = 0;

        if let Some(&(_idx, ch)) = self.chars.peek() {
            match ch {
                '\'' => {
                    // Empty char literal: ''
                    error_msg = Some("Empty character literal".to_string());
                    self.advance_char(); // consume the closing quote
                    closed = true;
                }
                '\\' => {
                    // Escape sequence
                    self.advance_char(); // consume backslash
                    if let Some(&(_escaped_idx, next_ch)) = self.chars.peek() {
                        let current_char_res = match next_ch {
                            'n' => Ok('\n'),
                            't' => Ok('\t'),
                            'r' => Ok('\r'),
                            '0' => Ok('\0'),
                            '\\' => Ok('\\'),
                            '\'' => Ok('\''),
                            // TODO: 'u' for \u{...}
                            _ => Err(format!(
                                "Invalid escape sequence in char literal: \\{}",
                                next_ch
                            )),
                        };
                        self.advance_char(); // consume the character after backslash

                        match current_char_res {
                            Ok(cv) => {
                                char_val = Some(cv);
                                consumed_char_count += 1;
                            }
                            Err(msg) => error_msg = Some(msg),
                        }
                    } else {
                        // Unterminated: EOF after backslash
                        error_msg =
                            Some("Unterminated character literal after backslash".to_string());
                        // lexeme will include the backslash
                    }
                }
                '\n' => {
                    // Unescaped newline
                    error_msg =
                        Some("Unterminated character literal (newline encountered)".to_string());
                    // Do not consume newline, it's not part of the error lexeme for the char
                }
                _ => {
                    // Regular character
                    char_val = Some(ch);
                    consumed_char_count += 1;
                    self.advance_char(); // consume the character
                }
            }
        } else {
            // Unterminated: EOF immediately after opening '
            error_msg = Some("Unterminated character literal (EOF)".to_string());
        }

        if error_msg.is_none() && !closed {
            // if no error so far, try to consume closing quote
            if let Some(&(_, ch)) = self.chars.peek() {
                if ch == '\'' {
                    self.advance_char(); // consume closing quote
                    closed = true;
                    if consumed_char_count > 1 {
                        error_msg = Some("Multi-character literal".to_string());
                    } else if consumed_char_count == 0 && char_val.is_none() {
                        // This case should ideally be caught by the empty check earlier,
                        // but as a safeguard if logic changes.
                        error_msg = Some("Empty character literal".to_string());
                    }
                } else {
                    // Expected closing quote, found something else or too many chars
                    if consumed_char_count >= 1 {
                        // If we consumed one char, now it's effectively multi-char or unclosed
                        error_msg = Some("Multi-character literal or unterminated".to_string());
                    } else {
                        error_msg = Some("Unterminated character literal".to_string());
                    }
                    // Do not consume the char if it's not a closing quote, let main loop handle it.
                }
            } else {
                // EOF before closing quote
                error_msg =
                    Some("Unterminated character literal (EOF before closing quote)".to_string());
            }
        }

        let current_lex_end_offset = self.current_offset();
        let current_lex_end_col = self.column;
        let current_lex_end_line = self.line;

        let final_lexeme = self
            .input
            .get(start_offset..current_lex_end_offset)
            .unwrap_or("");

        if let Some(msg) = error_msg {
            return Token {
                kind: TokenKind::Error,
                lexeme: final_lexeme.to_string(),
                literal: Some(LiteralValue::String(format!(
                    "{} (in character literal)",
                    msg
                ))),
                span: Span {
                    start: Position {
                        line: start_line,
                        column: start_col,
                        offset: start_offset,
                    },
                    end: Position {
                        line: current_lex_end_line,
                        column: current_lex_end_col,
                        offset: current_lex_end_offset,
                    },
                },
            };
        }

        if !closed {
            // This case should ideally be covered by error_msg logic, but as a fallback.
            return Token {
                kind: TokenKind::Error,
                lexeme: final_lexeme.to_string(),
                literal: Some(LiteralValue::String(
                    "Unterminated character literal (general)".to_string(),
                )),
                span: Span {
                    start: Position {
                        line: start_line,
                        column: start_col,
                        offset: start_offset,
                    },
                    end: Position {
                        line: current_lex_end_line,
                        column: current_lex_end_col,
                        offset: current_lex_end_offset,
                    },
                },
            };
        }

        // At this point, it should be a valid char literal with exactly one char_val.
        if let Some(cv) = char_val {
            if consumed_char_count == 1 {
                // Double check, though error_msg should catch multi-char or empty.
                Token {
                    kind: TokenKind::CharacterLiteral,
                    lexeme: final_lexeme.to_string(),
                    literal: Some(LiteralValue::Char(cv)),
                    span: Span {
                        start: Position {
                            line: start_line,
                            column: start_col,
                            offset: start_offset,
                        },
                        end: Position {
                            line: current_lex_end_line,
                            column: current_lex_end_col,
                            offset: current_lex_end_offset,
                        },
                    },
                }
            } else {
                // Should have been caught by error_msg logic for multi-char or empty.
                Token {
                    kind: TokenKind::Error,
                    lexeme: final_lexeme.to_string(),
                    literal: Some(LiteralValue::String(
                        "Invalid character literal state".to_string(),
                    )),
                    span: Span {
                        start: Position {
                            line: start_line,
                            column: start_col,
                            offset: start_offset,
                        },
                        end: Position {
                            line: current_lex_end_line,
                            column: current_lex_end_col,
                            offset: current_lex_end_offset,
                        },
                    },
                }
            }
        } else {
            // Should be caught by empty literal error check.
            Token {
                kind: TokenKind::Error,
                lexeme: final_lexeme.to_string(),
                literal: Some(LiteralValue::String(
                    "Empty character literal (final check)".to_string(),
                )),
                span: Span {
                    start: Position {
                        line: start_line,
                        column: start_col,
                        offset: start_offset,
                    },
                    end: Position {
                        line: current_lex_end_line,
                        column: current_lex_end_col,
                        offset: current_lex_end_offset,
                    },
                },
            }
        }
    }
}
