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
    ByteLiteral,      // e.g. b'a', b"foo"
    RawStringLiteral, // r"..."

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
        // Shebang handling: if input starts with "#!", skip the first line
        let input = if input.starts_with("#!") {
            match input.find('\n') {
                Some(idx) => &input[idx + 1..],
                None => "", // shebang is the whole file
            }
        } else {
            input
        };
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
                let mut current_indent = 0;
                let mut indent_char_type: Option<char> = None; // 's' for space, 't' for tab
                let mut mixed_indent_error = false;
                let indent_start_offset = self.chars.peek().map_or(0, |(i, _)| *i);
                let indent_start_col = self.column;

                // Consume leading whitespace to calculate indent
                while let Some(&(_, ch)) = self.chars.peek() {
                    if ch == ' ' {
                        if indent_char_type == Some('t') {
                            mixed_indent_error = true;
                        }
                        indent_char_type = Some('s');
                        current_indent += 1;
                        self.advance_char(); // Use normal advance_char
                    } else if ch == '\t' {
                        if indent_char_type == Some('s') {
                            mixed_indent_error = true;
                        }
                        indent_char_type = Some('t');
                        current_indent += 4; // Tabs are 4 spaces
                        self.advance_char();
                    } else {
                        break; // Not whitespace
                    }
                }

                if mixed_indent_error {
                    // Consume the rest of the mixed indent line up to non-whitespace or newline
                    let mut _error_lexeme_len = 0;
                    while let Some(&(_, ch_err)) = self.chars.peek() {
                        if ch_err != '\n' && ch_err.is_whitespace() {
                            _error_lexeme_len += ch_err.len_utf8();
                            self.advance_char();
                        } else {
                            break;
                        }
                    }
                    // The error lexeme is from indent_start_offset to current pos of self.chars
                    let error_lexeme = self
                        .input
                        .get(indent_start_offset..self.current_offset())
                        .unwrap_or("")
                        .to_string();

                    tokens.push(Token {
                        kind: TokenKind::Error,
                        lexeme: error_lexeme,
                        literal: Some(LiteralValue::String(
                            "Mixed tabs and spaces in indentation are not allowed.".to_string(),
                        )),
                        span: Span {
                            start: Position {
                                line: self.line,
                                column: indent_start_col,
                                offset: indent_start_offset,
                            },
                            end: Position {
                                line: self.line,
                                column: self.column,
                                offset: self.current_offset(),
                            },
                        },
                    });
                    // Skip to next line or handle rest of line as normal?
                    // For now, let's assume the error token covers the bad indent, and lexing continues.
                    // We need to ensure `at_line_start` is false now.
                    self.at_line_start = false; // Processed the start of the line (even if it was an error)
                } else {
                    // Original indentation logic
                    if current_indent > *self.indent_stack.last().unwrap() {
                        self.indent_stack.push(current_indent);
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
                                    column: current_indent + 1,
                                    offset: idx + current_indent,
                                },
                            },
                        });
                    } else if current_indent < *self.indent_stack.last().unwrap() {
                        while current_indent < *self.indent_stack.last().unwrap() {
                            self.indent_stack.pop();
                            self.pending_dedents += 1;
                        }
                    }
                    // Don't advance chars again - we already consumed leading whitespace in indentation calculation above
                    self.at_line_start = false;
                }
                // After indentation processing, re-peek to get the current character
                continue;
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
                        self.advance_char(); // Consume current char. THIS MUTATES THE REAL self.line/col
                        if c1 == '/' {
                            if let Some(&(_j, c2)) = self.chars.peek() {
                                if c2 == '*' {
                                    self.advance_char();
                                    nesting_level += 1;
                                }
                            }
                        } else if c1 == '*' {
                            if let Some(&(_j, c2)) = self.chars.peek() {
                                if c2 == '/' {
                                    self.advance_char();
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
                        let error_lexeme_str = self
                            .input
                            .get(comment_start_offset..self.current_offset())
                            .unwrap_or("");

                        let mut calc_end_line = comment_start_line;
                        let mut calc_end_col = comment_start_col;

                        for c in error_lexeme_str.chars() {
                            if c == '\n' {
                                calc_end_line += 1;
                                calc_end_col = 1;
                            } else {
                                calc_end_col += 1;
                            }
                        }

                        tokens.push(Token {
                            kind: TokenKind::Error,
                            lexeme: error_lexeme_str.to_string(),
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
                                    line: calc_end_line,
                                    column: calc_end_col,
                                    offset: self.current_offset(),
                                },
                            },
                        });
                    }
                    self.at_line_start = self.line != comment_start_line;
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
                if let Some((_, next_ch)) = self.peek_nth_char(1) {
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
                if let Some((_, next_ch)) = self.peek_nth_char(1) {
                    if next_ch == '\'' || next_ch == '"' {
                        tokens.push(self.lex_byte_literal(idx)); // Pass idx for start_offset
                        continue;
                    }
                }
            }

            // Raw String Literals: r"..."
            if ch == 'r' && self.peek_nth_char(1).is_some_and(|(_, c)| c == '"') {
                tokens.push(self.lex_raw_string_literal(idx));
                continue;
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

    // Helper function to parse \u{...} escape sequences
    // Consumes characters from self.chars.
    // Returns Ok(char) or Err(Token) if parsing fails (error token is fully formed).
    // Assumes '\\' and 'u' have already been consumed by the caller.
    // lit_start_offset, lit_start_line, lit_start_col are for the *entire literal* being parsed (e.g. string or char literal)
    // escape_start_offset, escape_start_line, escape_start_col are for the beginning of the \u sequence itself.
    fn parse_unicode_escape(
        &mut self,
        _lit_start_offset: usize,
        _lit_start_line: usize,
        _lit_start_col: usize,
        lit_kind: &str,
    ) -> Result<char, Token> {
        // 'u' has been consumed. current_offset points to char after 'u'. self.column is col after 'u'.
        let escape_u_offset = self.current_offset() - 'u'.len_utf8();
        let escape_u_line = self.line; // Line of 'u'
        let escape_u_col = self.column - 1; // Column of 'u'

        if self.chars.peek().is_some_and(|&(_, c)| c == '{') {
            self.advance_char(); // consume '{'
        } else {
            let err_tok_end_pos = Position {
                line: self.line,
                column: self.column,
                offset: self.current_offset(),
            };
            // Lexeme should be like \uX or \u<EOF>
            let err_lexeme = self
                .input
                .get(escape_u_offset - '\\'.len_utf8()..err_tok_end_pos.offset)
                .unwrap_or("\\u")
                .to_string();
            let found_char = self
                .chars
                .peek()
                .map(|(_, c)| c.to_string())
                .unwrap_or_else(|| "EOF".to_string());
            return Err(Token {
                kind: TokenKind::Error,
                lexeme: err_lexeme,
                literal: Some(LiteralValue::String(format!(
                    "Invalid Unicode escape in {} literal: expected '{{' after \\u, found '{}'.",
                    lit_kind, found_char
                ))),
                span: Span {
                    start: Position {
                        line: escape_u_line,
                        column: escape_u_col - 1,
                        offset: escape_u_offset - '\\'.len_utf8(),
                    }, // Span for \u sequence
                    end: err_tok_end_pos,
                },
            });
        }

        let mut hex_digits = String::new();
        let mut num_hex_digits = 0;
        let _hex_digits_start_offset = self.current_offset();
        let _hex_digits_start_col = self.column;

        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_ascii_hexdigit() {
                if num_hex_digits < 6 {
                    hex_digits.push(ch);
                    self.advance_char();
                    num_hex_digits += 1;
                } else {
                    // Too many hex digits
                    self.advance_char(); // Consume the char that makes it too long to include in lexeme
                    let err_tok_end_pos = Position {
                        line: self.line,
                        column: self.column,
                        offset: self.current_offset(),
                    };
                    let err_lexeme = self
                        .input
                        .get(escape_u_offset - '\\'.len_utf8()..err_tok_end_pos.offset)
                        .unwrap_or("")
                        .to_string();
                    return Err(Token {
                        kind: TokenKind::Error,
                        lexeme: err_lexeme,
                        literal: Some(LiteralValue::String(format!("Invalid Unicode escape in {} literal: too many hex digits (max 6) in \\u{{{}}}{{'.", lit_kind, hex_digits))),
                        span: Span { start: Position {line: escape_u_line, column: escape_u_col -1, offset: escape_u_offset - '\\'.len_utf8()}, end: err_tok_end_pos },
                    });
                }
            } else if ch == '}' {
                break;
            } else {
                // Invalid char in hex sequence
                let err_tok_end_pos = Position {
                    line: self.line,
                    column: self.column,
                    offset: self.current_offset(),
                };
                let err_lexeme = self
                    .input
                    .get(escape_u_offset - '\\'.len_utf8()..err_tok_end_pos.offset)
                    .unwrap_or("")
                    .to_string();
                return Err(Token {
                    kind: TokenKind::Error,
                    lexeme: err_lexeme,
                    literal: Some(LiteralValue::String(format!("Invalid Unicode escape in {} literal: unexpected character '{}' in \\u{{{}}} sequence.", lit_kind, ch, hex_digits))),
                    span: Span { start: Position {line: escape_u_line, column: escape_u_col -1, offset: escape_u_offset - '\\'.len_utf8()}, end: err_tok_end_pos },
                });
            }
        }

        if self.chars.peek().is_none_or(|&(_, c)| c != '}') {
            // Unterminated: missing '}'
            let err_tok_end_pos = Position {
                line: self.line,
                column: self.column,
                offset: self.current_offset(),
            };
            let err_lexeme = self
                .input
                .get(escape_u_offset - '\\'.len_utf8()..err_tok_end_pos.offset)
                .unwrap_or("")
                .to_string();
            return Err(Token {
                kind: TokenKind::Error,
                lexeme: err_lexeme,
                literal: Some(LiteralValue::String(format!("Invalid Unicode escape in {} literal: unclosed \\u{{{}}} sequence, missing '}}'.", lit_kind, hex_digits))),
                span: Span { start: Position {line: escape_u_line, column: escape_u_col-1, offset: escape_u_offset - '\\'.len_utf8()}, end: err_tok_end_pos },
            });
        }
        self.advance_char(); // consume '}'

        if num_hex_digits == 0 {
            let err_tok_end_pos = Position {
                line: self.line,
                column: self.column,
                offset: self.current_offset(),
            };
            let err_lexeme = self
                .input
                .get(escape_u_offset - '\\'.len_utf8()..err_tok_end_pos.offset)
                .unwrap_or("")
                .to_string();
            return Err(Token {
                kind: TokenKind::Error,
                lexeme: err_lexeme,
                literal: Some(LiteralValue::String(format!(
                    "Invalid Unicode escape in {} literal: empty hex code \\u{{}}.",
                    lit_kind
                ))),
                span: Span {
                    start: Position {
                        line: escape_u_line,
                        column: escape_u_col - 1,
                        offset: escape_u_offset - '\\'.len_utf8(),
                    },
                    end: err_tok_end_pos,
                },
            });
        }

        match u32::from_str_radix(&hex_digits, 16) {
            Ok(codepoint) => match std::char::from_u32(codepoint) {
                Some(ch_val) => Ok(ch_val),
                None => {
                    let err_tok_end_pos = Position {
                        line: self.line,
                        column: self.column,
                        offset: self.current_offset(),
                    };
                    let err_lexeme = self
                        .input
                        .get(escape_u_offset - '\\'.len_utf8()..err_tok_end_pos.offset)
                        .unwrap_or("")
                        .to_string();
                    Err(Token {
                        kind: TokenKind::Error,
                        lexeme: err_lexeme,
                        literal: Some(LiteralValue::String(format!("Invalid Unicode escape in {} literal: '\\u{{{}}}' is not a valid Unicode codepoint.", lit_kind, hex_digits))),
                        span: Span { start: Position {line: escape_u_line, column: escape_u_col-1, offset: escape_u_offset - '\\'.len_utf8()}, end: err_tok_end_pos },
                    })
                }
            },
            Err(_) => {
                // Should not happen if num_hex_digits > 0 and is_ascii_hexdigit passed
                let err_tok_end_pos = Position {
                    line: self.line,
                    column: self.column,
                    offset: self.current_offset(),
                };
                let err_lexeme = self
                    .input
                    .get(escape_u_offset - '\\'.len_utf8()..err_tok_end_pos.offset)
                    .unwrap_or("")
                    .to_string();
                Err(Token {
                    kind: TokenKind::Error,
                    lexeme: err_lexeme,
                    literal: Some(LiteralValue::String(format!(
                        "Internal error parsing hex '{}' for {} literal.",
                        hex_digits, lit_kind
                    ))),
                    span: Span {
                        start: Position {
                            line: escape_u_line,
                            column: escape_u_col - 1,
                            offset: escape_u_offset - '\\'.len_utf8(),
                        },
                        end: err_tok_end_pos,
                    },
                })
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
                '"' => {
                    self.advance_char(); // consume the closing quote
                    closed = true;
                    break;
                }
                '\\' => {
                    self.advance_char(); // consume backslash
                    if let Some(&(_escaped_idx, next_ch)) = self.chars.peek() {
                        match next_ch {
                            'n' => {
                                content.push('\n');
                                self.advance_char();
                            }
                            't' => {
                                content.push('\t');
                                self.advance_char();
                            }
                            '\\' => {
                                content.push('\\');
                                self.advance_char();
                            }
                            '"' => {
                                content.push('"');
                                self.advance_char();
                            }
                            'u' => {
                                self.advance_char();
                                match self.parse_unicode_escape(
                                    start_offset,
                                    start_line,
                                    start_col,
                                    "string",
                                ) {
                                    Ok(uc) => content.push(uc),
                                    Err(token) => return token,
                                }
                            }
                            _ => {
                                let specific_error_lexeme = format!("\\{}", next_ch);
                                self.advance_char();
                                let error_token_lexeme = self
                                    .input
                                    .get(start_offset..self.current_offset())
                                    .unwrap_or("")
                                    .to_string();
                                return Token {
                                    kind: TokenKind::Error,
                                    lexeme: error_token_lexeme,
                                    literal: Some(LiteralValue::String(format!(
                                        "Invalid escape sequence in string literal: {}. Only valid escapes are \\n, \\t, \\\\, \\\" and \\u{{...}}.",
                                        specific_error_lexeme
                                    ))),
                                    span: Span {
                                        start: Position { line: start_line, column: start_col, offset: start_offset },
                                        end: Position { line: self.line, column: self.column, offset: self.current_offset() },
                                    },
                                };
                            }
                        }
                    } else {
                        // Unterminated escape sequence at EOF
                        let current_lex_end_offset = self.current_offset();
                        let current_lex_end_col = self.column;
                        let current_lex_end_line = self.line;
                        let lexeme = self
                            .input
                            .get(start_offset..current_lex_end_offset)
                            .unwrap_or("")
                            .to_string();
                        return Token {
                            kind: TokenKind::Error,
                            lexeme,
                            literal: Some(LiteralValue::String(
                                "Unterminated escape sequence at end of string literal: expected character after \\".to_string(),
                            )),
                            span: Span {
                                start: Position { line: start_line, column: start_col, offset: start_offset },
                                end: Position { line: current_lex_end_line, column: current_lex_end_col, offset: current_lex_end_offset },
                            },
                        };
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
                    r#"Unterminated string literal: expected closing quote " before end of line or file."#.to_string(),
                )),
                span: Span {
                    start: Position { line: start_line, column: start_col, offset: start_offset },
                    end: Position { line: current_lex_end_line, column: current_lex_end_col, offset: current_lex_end_offset },
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
                    line: current_lex_end_line,
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
                    error_msg = Some("Empty character literal ".to_string());
                    self.advance_char(); // consume the closing quote
                    closed = true;
                }
                '\\' => {
                    // Escape sequence
                    self.advance_char(); // consume backslash
                    if let Some(&(_escaped_idx, next_ch)) = self.chars.peek() {
                        if next_ch == 'u' {
                            self.advance_char(); // consume 'u'
                            match self.parse_unicode_escape(
                                start_offset,
                                start_line,
                                start_col,
                                "character",
                            ) {
                                Ok(cv) => {
                                    char_val = Some(cv);
                                    consumed_char_count += 1;
                                    // After a valid unicode escape, check for closing quote
                                    if let Some(&(_, ch)) = self.chars.peek() {
                                        if ch == '\'' {
                                            self.advance_char(); // consume closing quote
                                            let cv = char_val.unwrap();
                                            let final_lexeme = self
                                                .input
                                                .get(start_offset..self.current_offset())
                                                .unwrap_or("");
                                            return Token {
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
                                                        line: self.line,
                                                        column: self.column,
                                                        offset: self.current_offset(),
                                                    },
                                                },
                                            };
                                        } else {
                                            // Not a closing quote: multi-character or unterminated
                                            error_msg = Some("Multi-character literal or unterminated (in character literal)".to_string());
                                        }
                                    } else {
                                        // EOF before closing quote
                                        error_msg = Some("Unterminated character literal (EOF before closing quote) (in character literal)".to_string());
                                    }
                                }
                                Err(token) => return token, // Return error token directly
                            }
                        } else {
                            // Simple escape like \n, \t, etc.
                            let current_char_res = match next_ch {
                                'n' => Ok('\n'),
                                't' => Ok('\t'),
                                'r' => Ok('\r'),
                                '0' => Ok('\0'),
                                '\\' => Ok('\\'),
                                '\'' => Ok('\''),
                                _ => Err(format!(
                                    "Invalid escape sequence in char literal: \\{}",
                                    next_ch
                                )),
                            };
                            self.advance_char(); // consume the character after backslash (e.g. 'n' in \n)

                            match current_char_res {
                                Ok(cv) => {
                                    char_val = Some(cv);
                                    consumed_char_count += 1;
                                }
                                // Error message for char lit needs to be specific for invalid simple escape
                                Err(msg_str) => {
                                    error_msg = Some(format!("{} (in character literal)", msg_str))
                                }
                            }
                        }
                    } else {
                        // Unterminated: EOF after backslash (e.g. '\')
                        error_msg = Some(
                            "Unterminated character literal after backslash (in character literal)"
                                .to_string(),
                        );
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
                        error_msg = Some("Multi-character literal ".to_string());
                    } else if consumed_char_count == 0 && char_val.is_none() {
                        error_msg = Some("Empty character literal ".to_string());
                    }
                } else {
                    // Expected closing quote, found something else or too many chars
                    if consumed_char_count >= 1 {
                        error_msg = Some("Multi-character literal or unterminated ".to_string());
                    } else {
                        error_msg = Some("Unterminated character literal ".to_string());
                    }
                }
            } else {
                // EOF before closing quote
                error_msg =
                    Some("Unterminated character literal (EOF before closing quote) ".to_string());
            }
        }

        let current_lex_end_offset = self.current_offset();
        let current_lex_end_col = self.column;
        let current_lex_end_line = self.line;

        let final_lexeme = self
            .input
            .get(start_offset..current_lex_end_offset)
            .unwrap_or("");

        if let Some(mut msg) = error_msg {
            if !msg.ends_with("(in character literal)") {
                if msg.ends_with(' ') {
                    msg.push_str("(in character literal)");
                } else {
                    msg.push_str(" (in character literal)");
                }
            }
            return Token {
                kind: TokenKind::Error,
                lexeme: final_lexeme.to_string(),
                literal: Some(LiteralValue::String(msg)),
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
        // If we have an error, return an Error token
        if !closed {
            return Token {
                kind: TokenKind::Error,
                lexeme: final_lexeme.to_string(),
                literal: Some(LiteralValue::String(
                    "Unterminated character literal (in character literal)".to_string(),
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
                            line: self.line,
                            column: self.column,
                            offset: self.current_offset(),
                        },
                    },
                }
            } else {
                Token {
                    kind: TokenKind::Error,
                    lexeme: final_lexeme.to_string(),
                    literal: Some(LiteralValue::String(
                        "Multi-character literal or unterminated (in character literal)"
                            .to_string(),
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
            Token {
                kind: TokenKind::Error,
                lexeme: final_lexeme.to_string(),
                literal: Some(LiteralValue::String(
                    "Unterminated character literal (in character literal)".to_string(),
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

    fn lex_raw_string_literal(&mut self, start_idx: usize) -> Token {
        let start_line = self.line;
        let start_col = self.column;

        self.advance_char(); // consume 'r'
        self.advance_char(); // consume '"'

        let content_start_offset = self.current_offset();
        let mut content = String::new();
        let mut closed = false;

        while let Some(&(_i, ch)) = self.chars.peek() {
            if ch == '"' {
                // Closing quote for raw string
                // Check if it's followed by another quote (for r""" style, not supported yet)
                // For r"...", this is the end.
                self.advance_char(); // consume closing '"'
                closed = true;
                break;
            }
            // No escape processing for raw strings
            content.push(ch);
            self.advance_char();
        }
        let end_offset = self.current_offset();

        let span = Span {
            start: Position {
                line: start_line,
                column: start_col,
                offset: start_idx,
            },
            end: Position {
                line: self.line,
                column: self.column,
                offset: end_offset,
            },
        };

        if !closed {
            Token {
                kind: TokenKind::Error,
                lexeme: self.input.get(start_idx..end_offset).unwrap_or("").to_string(),
                literal: Some(LiteralValue::String("Unterminated raw string literal: expected closing quote \" before end of line or file.".to_string())),
                span,
            }
        } else {
            // Extract the content without the r" and " delimiters
            let actual_content = self
                .input
                .get(content_start_offset..(end_offset - '"'.len_utf8()))
                .unwrap_or("");
            Token {
                kind: TokenKind::RawStringLiteral,
                lexeme: self
                    .input
                    .get(start_idx..end_offset)
                    .unwrap_or("")
                    .to_string(),
                literal: Some(LiteralValue::String(actual_content.to_string())),
                span,
            }
        }
    }

    fn lex_byte_literal(&mut self, start_offset_param: usize) -> Token {
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
                // Basic escape handling for byte strings (e.g., b"\\", b"\'", b"\"")
                self.advance_char(); // consume backslash
                if let Some(&(_, esc_char)) = self.chars.peek() {
                    match esc_char {
                        'n' | 't' | 'r' | '0' | '\\' | '\'' | '"' => {
                            // Common escapes, store as is
                            content.push('\\');
                            content.push(esc_char);
                        }
                        // Hex escapes like \xHH could be added here if desired for byte strings
                        _ => {
                            // For byte strings, unknown escapes might be stored literally or be an error
                            // Storing literally: \ and the char
                            content.push('\\');
                            content.push(esc_char);
                        }
                    }
                    self.advance_char(); // consume the escaped char
                } else {
                    break; // Unterminated escape at EOF
                }
            } else {
                content.push(c);
                self.advance_char();
            }
        }
        let end_offset = self.current_offset();
        let lexeme = self.input.get(start_offset_param..end_offset).unwrap_or("");
        let span = Span {
            start: Position {
                line: start_line,
                column: start_col,
                offset: start_offset_param,
            },
            end: Position {
                line: self.line,
                column: self.column,
                offset: end_offset,
            },
        };

        if !closed {
            Token {
                kind: TokenKind::Error,
                lexeme: lexeme.to_string(),
                literal: Some(LiteralValue::String(
                    "Unterminated byte literal".to_string(),
                )),
                span,
            }
        } else if quote == '\'' {
            // Single-quoted byte literal, b'...'
            // As per RFC-001 (implicitly, via char literal def), char/byte literals are single char/byte.
            // For b'...', the `content` after escape processing must be a single byte representation.
            // This requires more robust escape processing if e.g. b'\n' is to become a single byte 10.
            // Current simple escape stores \ and n. For b'\n', content is "\n".len() = 2.
            // For now, if it's b'c', content is "c". If b'\\ ', content is "\\".
            // A simple check: if content contains \, it's likely an escape not yet processed to a single byte.
            // Or, if content.chars().count() != 1 for simple chars.
            // Let's refine this: Ferra design implies byte literals are simple for now.
            // b'a' -> byte 'a'. b'\'' -> byte '''. b'\\' -> byte '\\'.
            // What about b'\n'? The current escape logic makes `content` = "\n".
            // This is tricky for b'X'. For now, assume content should be 1 char unless it's a known single-byte escape.

            if content.chars().count() == 1 {
                Token {
                    kind: TokenKind::ByteLiteral,
                    lexeme: lexeme.to_string(),
                    literal: Some(LiteralValue::Byte(content.as_bytes()[0])),
                    span,
                }
            } else {
                Token {
                    kind: TokenKind::Error,
                    lexeme: lexeme.to_string(),
                    literal: Some(LiteralValue::String(
                        "Byte literal b'...' must represent a single byte after escapes."
                            .to_string(),
                    )),
                    span,
                }
            }
        } else {
            Token {
                kind: TokenKind::ByteLiteral,
                lexeme: lexeme.to_string(),
                literal: Some(LiteralValue::String(content)),
                span,
            }
        }
    }
}
