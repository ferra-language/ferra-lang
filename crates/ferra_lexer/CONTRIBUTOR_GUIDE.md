# Ferra Lexer Contributor Guide

**Version**: 1.0  
**Last Updated**: January 2025  
**Phase 1 Complete**: Lexer development framework and contribution guidelines  

---

## Overview

This guide provides comprehensive instructions for contributing to the Ferra Lexer crate. The lexer is the first stage of the Ferra compiler pipeline, responsible for converting source code into tokens with comprehensive error reporting and Unicode support.

### Project Status
- **Phase 1 Complete**: All core lexical analysis features implemented
- **116 Tests Total**: (0 unit tests + 116 integration tests)
- **Production Ready**: Unicode-aware, high-performance lexing
- **Stable API**: Well-defined token stream interface for parser integration

---

## Getting Started

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required tools
cargo install cargo-tarpaulin  # Code coverage
cargo install criterion        # Benchmarking
cargo install cargo-audit      # Security auditing

# Clone repository
git clone https://github.com/ferra-lang/ferra-lang.git
cd ferra-lang/crates/ferra_lexer
```

### Development Setup

```bash
# Install development dependencies
cargo build --all-features
cargo test --all

# Run full test suite
cargo test --all-targets
cargo test --doc

# Check formatting and linting
cargo fmt --check
cargo clippy --all-targets -- -D warnings

# Run benchmarks
cargo bench # Note: Requires benches/ directory and benchmark targets to be defined.
```

---

## Architecture Overview

### Core Components

```
ferra_lexer/
├── src/
│   ├── token/         # Token type definitions (logical module)
│   ├── lexer/         # Core lexing logic (logical module)
│   ├── error/         # Error types and handling (logical module)
│   ├── unicode/       # Unicode support utilities (logical module)
│   ├── keywords/      # Keyword recognition (logical module)
│   └── lib.rs         # Public API surface and main module
├── tests/             # Integration tests (116 tests in `tests/` directory)
# Note: `benches/`, `examples/`, and `docs/` specific to `ferra_lexer` are not currently present directly under `crates/ferra_lexer/`.
# Project-level documentation may be in the root `/docs` directory.
# Benchmarks and examples might be managed at the workspace level.
```

### Key Abstractions

#### 1. Token Type System
- **Purpose**: Comprehensive representation of all Ferra language tokens
- **Implementation**: Enum-based with value preservation for literals
- **Features**: Source span tracking, detailed token information

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    BooleanLiteral(bool),
    
    // Keywords
    Fn, Let, Var, If, Else, // ...
    
    // Operators and Punctuation
    Plus, Minus, Star, Slash, // ...
    
    // Special
    Eof, Newline, Comment(String),
}
```

#### 2. Lexer State Machine
- **Purpose**: Efficient character-by-character processing
- **Pattern**: Iterator-based with lookahead capability
- **Features**: Error recovery, Unicode handling, position tracking

#### 3. Source Location Tracking
- **Purpose**: Precise error reporting and IDE integration
- **Implementation**: Byte and character position tracking
- **Features**: Line/column conversion, span calculation

#### 4. Error System
- **Purpose**: Detailed lexical error reporting
- **Strategy**: Continue lexing after errors when possible
- **Goal**: Helpful error messages with context

---

## Development Workflow

### Feature Development

1. **Create Feature Branch**
```bash
git checkout -b feat/your-lexer-feature
```

2. **Write Tests First** (TDD approach)
```bash
# Add test case to appropriate test file
# tests/test_your_feature.rs

#[test]
fn test_your_feature_basic() {
    let source = "your test input";
    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens[0].token_type, TokenType::YourExpectedToken);
}
```

3. **Implement Feature**
```bash
# Implement in relevant src/ module
# Follow existing patterns and conventions
```

4. **Run Test Suite**
```bash
cargo test your_feature
cargo test --all  # Full test suite
```

5. **Check Performance Impact**
```bash
cargo bench  # Run benchmarks (Requires benches/ directory and benchmark targets to be defined)
# Check for performance regressions
```

6. **Update Documentation**
```bash
# Update relevant .md files
# Add examples if needed
cargo doc --open  # Check generated docs
```

### Commit Guidelines

Follow Conventional Commits specification:

```bash
# Feature commits
git commit -m "feat: add support for hexadecimal integer literals"

# Bug fixes
git commit -m "fix: handle Unicode escape sequences correctly"

# Performance improvements
git commit -m "perf: optimize string literal parsing"

# Documentation
git commit -m "docs: update lexer API documentation"

# Tests
git commit -m "test: add comprehensive numeric literal test coverage"
```

---

## Lexer-Specific Development Guidelines

### Token Type Development

#### Adding New Token Types

1. **Define Token Variant**
```rust
// In src/token/mod.rs
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // existing variants...
    YourNewToken(OptionalValue),
}
```

2. **Add Recognition Logic**
```rust
// In src/lexer/mod.rs
impl Lexer {
    fn scan_token(&mut self) -> Result<Token, LexError> {
        match self.current_char() {
            // existing cases...
            'your_char' => self.scan_your_new_token(),
            _ => self.scan_unexpected_character(),
        }
    }
    
    fn scan_your_new_token(&mut self) -> Result<Token, LexError> {
        let start = self.position;
        // Implementation
        let span = Span::new(start, self.position, self.file_name.clone());
        Ok(Token::new(TokenType::YourNewToken(value), span))
    }
}
```

3. **Add Test Coverage**
```rust
#[test]
fn test_your_new_token() {
    let source = "your_token_syntax";
    let tokens = tokenize(source).unwrap();
    
    assert_eq!(tokens.len(), 2); // token + EOF
    assert_eq!(tokens[0].token_type, TokenType::YourNewToken(expected_value));
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, source.len());
}
```

4. **Update Token Display**
```rust
// In src/token/display.rs
impl Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // existing cases...
            TokenType::YourNewToken(value) => write!(f, "your_token({:?})", value),
        }
    }
}
```

### Keyword Development

#### Adding New Keywords

1. **Define Keyword**
```rust
// In src/keywords/mod.rs
pub const KEYWORDS: &[(&str, TokenType)] = &[
    // existing keywords...
    ("your_keyword", TokenType::YourKeyword),
];
```

2. **Add to Recognition**
```rust
// Keywords are automatically recognized by keyword table lookup
// No additional implementation needed
```

3. **Add Tests**
```rust
#[test]
fn test_your_keyword() {
    let source = "your_keyword";
    let tokens = tokenize(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::YourKeyword);
}

#[test]
fn test_keyword_vs_identifier() {
    let source = "your_keyword_extended"; // Should be identifier
    let tokens = tokenize(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::Identifier("your_keyword_extended".to_string()));
}
```

### Literal Parsing Development

#### Adding New Literal Types

1. **Define Literal Token**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // existing literals...
    YourLiteralType(YourValueType),
}
```

2. **Implement Parsing Logic**
```rust
impl Lexer {
    fn scan_your_literal(&mut self) -> Result<Token, LexError> {
        let start = self.position;
        let mut value_builder = String::new();
        
        // Parse literal syntax
        while !self.is_at_end() && self.is_your_literal_char() {
            value_builder.push(self.advance());
        }
        
        // Convert to appropriate type
        let value = self.parse_your_literal_value(&value_builder)?;
        
        let span = Span::new(start, self.position, self.file_name.clone());
        Ok(Token::new(TokenType::YourLiteralType(value), span))
    }
    
    fn parse_your_literal_value(&self, text: &str) -> Result<YourValueType, LexError> {
        // Implementation with error handling
        text.parse().map_err(|_| {
            LexError::invalid_literal(
                format!("Invalid {} literal: {}", "your_type", text),
                self.current_span(),
            )
        })
    }
}
```

3. **Add Comprehensive Tests**
```rust
#[test]
fn test_your_literal_valid_cases() {
    let test_cases = vec![
        ("valid_syntax_1", expected_value_1),
        ("valid_syntax_2", expected_value_2),
        // More test cases
    ];
    
    for (source, expected) in test_cases {
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token_type, TokenType::YourLiteralType(expected));
    }
}

#[test]
fn test_your_literal_error_cases() {
    let error_cases = vec![
        "invalid_syntax_1",
        "invalid_syntax_2",
    ];
    
    for source in error_cases {
        let result = tokenize(source);
        assert!(result.is_err(), "Expected error for: {}", source);
    }
}
```

### Error Handling Development

#### Adding New Error Types

1. **Define Error Variant**
```rust
// In src/error/mod.rs
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LexError {
    // existing variants...
    
    #[error("Your error description: {message}")]
    YourNewError {
        message: String,
        span: Span,
    },
}
```

2. **Implement Error Construction**
```rust
impl LexError {
    pub fn your_new_error(message: String, span: Span) -> Self {
        Self::YourNewError { message, span }
    }
}
```

3. **Add Error Recovery**
```rust
impl Lexer {
    fn handle_your_error(&mut self, error: LexError) -> Option<Token> {
        // Log or collect error
        self.errors.push(error);
        
        // Attempt recovery
        self.skip_to_safe_point();
        
        // Continue lexing or return None
        None
    }
}
```

---

## Testing Guidelines

### Test Structure

#### Unit Tests (in `src/`)
- **Purpose**: Test individual lexer components
- **Location**: `#[cfg(test)] mod tests` in each module
- **Focus**: Token recognition, Unicode handling, error cases

#### Integration Tests (in `tests/`)
- **Purpose**: Test complete lexing scenarios
- **Organization**: One file per feature area
- **Coverage**: All token types and language constructs

#### Benchmarks (in `benches/`)
- **Purpose**: Performance monitoring
- **Categories**: Tokenization speed, memory usage
- **Baseline**: Performance expectations

### Writing Effective Tests

#### Test Naming Convention
```rust
// Unit tests: test_{component}_{scenario}
#[test]
fn test_string_literal_escape_sequences() { }

// Integration tests: test_{feature}_{scenario}
#[test]
fn test_numeric_literals_comprehensive() { }

// Benchmarks: {category}_{specific_case}
fn tokenization_large_file(c: &mut Criterion) { }
```

#### Test Patterns

**Token Recognition Pattern:**
```rust
#[test]
fn test_token_recognition() {
    let test_cases = vec![
        ("input1", TokenType::ExpectedToken1),
        ("input2", TokenType::ExpectedToken2),
    ];
    
    for (source, expected) in test_cases {
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens.len(), 2); // token + EOF
        assert_eq!(tokens[0].token_type, expected);
    }
}
```

**Error Handling Pattern:**
```rust
#[test]
fn test_error_scenarios() {
    let error_cases = vec![
        ("invalid_input1", "expected error message"),
        ("invalid_input2", "another error message"),
    ];
    
    for (source, expected_msg) in error_cases {
        match tokenize(source) {
            Err(LexError::YourExpectedError { message, .. }) => {
                assert!(message.contains(expected_msg));
            }
            other => panic!("Expected YourExpectedError for '{}', got {:?}", source, other),
        }
    }
}
```

**Unicode Handling Pattern:**
```rust
#[test]
fn test_unicode_support() {
    let unicode_cases = vec![
        ("café", TokenType::Identifier("café".to_string())),
        ("🦀", TokenType::Identifier("🦀".to_string())),
        ("αβγ", TokenType::Identifier("αβγ".to_string())),
    ];
    
    for (source, expected) in unicode_cases {
        let tokens = tokenize(source).unwrap();
        assert_eq!(tokens[0].token_type, expected);
    }
}
```

### Performance Testing

#### Adding Benchmarks

```rust
// In benches/lexer_benchmarks.rs (Note: This directory needs to be created for ferra_lexer)
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn tokenization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenization");
    
    for size in &[1_000, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::new("file_size", size),
            size,
            |b, &size| {
                let source = generate_test_source(size);
                
                b.iter(|| {
                    ferra_lexer::tokenize(&source).unwrap()
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, tokenization_benchmark);
criterion_main!(benches);
```

---

## Code Quality Standards

### Code Style

#### Character Processing
```rust
// Good: Use iterator patterns
impl Lexer {
    fn scan_identifier(&mut self) -> Token {
        let start = self.position;
        
        while self.current_char().is_alphanumeric() || self.current_char() == '_' {
            self.advance();
        }
        
        let text = self.source[start..self.position].to_string();
        let token_type = self.keyword_or_identifier(text);
        
        Token::new(token_type, Span::new(start, self.position, self.file_name.clone()))
    }
}

// Good: Unicode-aware character handling
fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
```

#### Error Handling
```rust
// Good: Structured error handling
fn scan_string_literal(&mut self) -> Result<Token, LexError> {
    let start = self.position;
    self.advance(); // Skip opening quote
    
    let mut value = String::new();
    
    while !self.is_at_end() && self.current_char() != '"' {
        if self.current_char() == '\\' {
            value.push(self.scan_escape_sequence()?);
        } else {
            value.push(self.advance());
        }
    }
    
    if self.is_at_end() {
        return Err(LexError::unterminated_string(
            "Unterminated string literal".to_string(),
            Span::new(start, self.position, self.file_name.clone()),
        ));
    }
    
    self.advance(); // Skip closing quote
    
    Ok(Token::new(
        TokenType::StringLiteral(value),
        Span::new(start, self.position, self.file_name.clone()),
    ))
}
```

### Performance Guidelines

#### Efficient Character Processing
```rust
// Good: Minimize allocations
impl Lexer {
    fn scan_number(&mut self) -> Token {
        let start = self.position;
        
        // Use slice instead of building string
        while self.current_char().is_ascii_digit() {
            self.advance();
        }
        
        // Only allocate when needed
        let text = &self.source[start..self.position];
        let value = text.parse::<i64>().unwrap();
        
        Token::new(
            TokenType::IntegerLiteral(value),
            Span::new(start, self.position, self.file_name.clone())
        )
    }
}

// Good: Efficient keyword lookup
lazy_static! {
    static ref KEYWORD_MAP: HashMap<&'static str, TokenType> = {
        KEYWORDS.iter().cloned().collect()
    };
}

fn keyword_or_identifier(&self, text: String) -> TokenType {
    KEYWORD_MAP.get(text.as_str())
        .cloned()
        .unwrap_or_else(|| TokenType::Identifier(text))
}
```

---

## Common Patterns

### Character Processing Pattern

```rust
impl Lexer {
    /// Consume characters while predicate is true
    fn consume_while<F>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let start = self.position;
        
        while !self.is_at_end() && predicate(self.current_char()) {
            self.advance();
        }
        
        self.source[start..self.position].to_string()
    }
    
    /// Peek ahead n characters
    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.source.chars().nth(self.position + n)
    }
    
    /// Check if current position matches string
    fn matches_string(&self, text: &str) -> bool {
        self.source[self.position..].starts_with(text)
    }
}
```

### Error Recovery Pattern

```rust
impl Lexer {
    /// Skip to next valid token start
    fn skip_to_token_boundary(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                    break;
                }
                c if c.is_alphabetic() || "(){}[]".contains(c) => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
    
    /// Continue lexing after error
    fn recover_from_error(&mut self, error: LexError) -> Vec<Token> {
        self.errors.push(error);
        self.skip_to_token_boundary();
        self.scan_remaining_tokens()
    }
}
```

---

## Debugging Guidelines

### Lexer Debugging

#### Debug Utilities
```rust
#[cfg(debug_assertions)]
impl Lexer {
    fn debug_state(&self) -> String {
        format!(
            "Lexer at position {}: current_char={:?}, line={}, column={}",
            self.position,
            self.current_char(),
            self.line,
            self.column
        )
    }
    
    fn debug_token(&self, token: &Token) -> String {
        format!(
            "Token: {:?} at {}:{}",
            token.token_type,
            token.span.start,
            token.span.end
        )
    }
}
```

#### Test Debugging
```rust
#[test]
fn debug_tokenization() {
    let source = "problematic input";
    
    // Enable detailed logging
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    let mut lexer = Lexer::new(source, "test.ferra");
    
    loop {
        println!("Lexer state: {}", lexer.debug_state());
        
        match lexer.next_token() {
            Ok(token) => {
                println!("Generated: {}", lexer.debug_token(&token));
                if token.token_type == TokenType::Eof {
                    break;
                }
            }
            Err(error) => {
                println!("Error: {:?}", error);
                break;
            }
        }
    }
}
```

---

## Integration with Parser

### Token Stream Interface

The lexer provides tokens to the parser through a well-defined interface:

```rust
// Lexer produces tokens
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source, "<input>");
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token()?;
        let is_eof = token.token_type == TokenType::Eof;
        tokens.push(token);
        
        if is_eof {
            break;
        }
    }
    
    Ok(tokens)
}

// Parser consumes tokens via TokenStream trait
impl TokenStream for VecTokenStream {
    fn peek(&self) -> &Token { /* ... */ }
    fn consume(&mut self) -> Token { /* ... */ }
    fn is_at_end(&self) -> bool { /* ... */ }
}
```

### Error Coordination

```rust
// Lexer errors are reported to parser
pub enum CompilerError {
    LexError(LexError),
    ParseError(ParseError),
}

// Error spans coordinate between lexer and parser
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,    // Byte position
    pub end: usize,      // Byte position
    pub file: String,    // Source file name
}
```

---

## Release Process

### Pre-Release Checklist

1. **All Tests Pass**
```bash
cargo test --all-targets
cargo test --doc
cargo bench
```

2. **Unicode Compliance**
```bash
# Test with various Unicode inputs
cargo test unicode
```

3. **Performance Baseline**
```bash
cargo bench > lexer_baseline.txt
```

4. **Integration Testing**
```bash
# Test with parser integration
cd ../ferra_parser
cargo test integration
```

---

## Getting Help

### Resources
- **Design Documentation**: `DESIGN_IMPLEMENTATION_PLAN.md`
- **Token Reference**: `src/token/mod.rs`
- **Unicode Handling**: `src/unicode/mod.rs`

### Community
- **Discussions**: GitHub Discussions for questions
- **Issues**: GitHub Issues for bugs and feature requests

This contributor guide ensures consistent, high-quality contributions to the Ferra Lexer while maintaining Unicode compliance and optimal performance. 