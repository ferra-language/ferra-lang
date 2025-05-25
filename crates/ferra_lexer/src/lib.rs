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
                    // Block comment (no nesting)
                    self.advance_char(); // consume '/'
                    self.advance_char(); // consume '*'
                    let mut closed = false;
                    while let Some(&(_i, c1)) = self.chars.peek() {
                        self.advance_char();
                        if c1 == '*' {
                            if let Some(&(_j, c2)) = self.chars.peek() {
                                if c2 == '/' {
                                    self.advance_char(); // consume '/'
                                    closed = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !closed {
                        tokens.push(Token {
                            kind: TokenKind::Error,
                            lexeme: self.input[idx..].to_string(),
                            literal: Some(LiteralValue::String(
                                "Unterminated block comment".to_string(),
                            )),
                            span: Span {
                                start: Position {
                                    line: self.line,
                                    column: self.column,
                                    offset: idx,
                                },
                                end: Position {
                                    line: self.line,
                                    column: self.column,
                                    offset: self.current_offset(),
                                },
                            },
                        });
                    }
                    continue;
                }
            }

            // Handle Newlines
            if ch == '\n' {
                let start_offset = idx;
                let start_col = self.column;
                self.advance_char(); // Consumes the newline, updates line and column
                tokens.push(Token {
                    kind: TokenKind::Newline,
                    lexeme: "\n".to_string(),
                    literal: None,
                    span: Span {
                        start: Position {
                            line: self.line - 1, // Line before advancing
                            column: start_col,
                            offset: start_offset,
                        },
                        end: Position {
                            line: self.line - 1,   // Line before advancing
                            column: start_col + 1, // Newline takes one char column conceptually before reset
                            offset: start_offset + 1,
                        },
                    },
                });
                continue;
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

            // Identifiers and keywords: [a-zA-Z_][a-zA-Z0-9_]*
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = idx;
                let start_col = self.column;
                let mut end_col = self.column;
                let mut ident = String::new();
                while let Some(&(_, c)) = self.chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        ident.push(c);
                        let (_j, _) = self.advance_char().unwrap();
                        end_col = self.column;
                    } else {
                        break;
                    }
                }
                let end_offset = self.current_offset();
                let kind = match ident.as_str() {
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
                    lexeme: ident,
                    literal: literal_value,
                    span: Span {
                        start: Position {
                            line: self.line,
                            column: start_col,
                            offset: start,
                        },
                        end: Position {
                            line: self.line,
                            column: end_col,
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
                if self.input[idx..].starts_with(op) {
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
                            '"' => content.push('"'),
                            // TODO: Add \u{...} handling here as per DESIGN_LEXER.md
                            _ => {
                                // Invalid escape: emit error token
                                let current_lex_end_offset = self.current_offset();
                                let current_lex_end_col = self.column;
                                let current_lex_end_line = self.line;
                                let lexeme = self.input[start_offset..current_lex_end_offset + 1]
                                    .to_string();
                                self.advance_char(); // consume the invalid escape char
                                return Token {
                                    kind: TokenKind::Error,
                                    lexeme,
                                    literal: Some(LiteralValue::String(format!(
                                        "Invalid escape sequence in string literal: \\{}",
                                        next_ch
                                    ))),
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
            // The string was not properly closed. It could be EOF or an unescaped newline.
            // The lexeme includes the opening quote up to the point of termination.
            // The current_offset is correct here as it's where we stopped.
            return Token {
                kind: TokenKind::Error,
                lexeme: self.input[start_offset..current_lex_end_offset].to_string(),
                literal: Some(LiteralValue::String(
                    "Unterminated string literal".to_string(),
                )),
                span: Span {
                    start: Position {
                        line: start_line,
                        column: start_col,
                        offset: start_offset,
                    },
                    end: Position {
                        // The end is where the lexer stopped processing this token.
                        line: current_lex_end_line,
                        column: current_lex_end_col,
                        offset: current_lex_end_offset,
                    },
                },
            };
        }

        Token {
            kind: TokenKind::StringLiteral,
            lexeme: self.input[start_offset..current_lex_end_offset].to_string(),
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

        let final_lexeme = self.input[start_offset..current_lex_end_offset].to_string();

        if let Some(msg) = error_msg {
            return Token {
                kind: TokenKind::Error,
                lexeme: final_lexeme,
                literal: Some(LiteralValue::String(msg)), // Using String for error messages
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
                lexeme: final_lexeme,
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
                    lexeme: final_lexeme,
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
                    lexeme: final_lexeme,
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
                lexeme: final_lexeme,
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
