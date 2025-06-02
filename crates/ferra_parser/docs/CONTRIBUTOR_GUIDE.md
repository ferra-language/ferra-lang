# Ferra Parser Contributor Guide

**Version**: 1.0  
**Last Updated**: January 2025  
**Phase 2 Complete**: Parser development framework and contribution guidelines  

---

## Overview

This guide provides comprehensive instructions for contributing to the Ferra Parser crate. The parser is a critical component that converts Ferra source code into Abstract Syntax Trees (AST) with robust error recovery and performance optimization.

### Project Status
- **Phase 2 Complete**: All core parsing features implemented
- **429 Parser Tests**: (67 unit + 362 integration) covering language features
- **Production Ready**: Battle-tested with performance optimizations
- **Active Development**: Continuous improvements and feature additions

### Test Suite: 429 Tests Total (67 unit + 362 integration)

The parser test suite has grown significantly with comprehensive coverage:

**Core Features Testing:**
- **Expression Parsing**: 27 comprehensive tests covering all operators and precedence
- **Statement Parsing**: 13 tests for declarations and control flow  
- **Block Structures**: 30 tests for braced/indented blocks with scope management
- **Type System**: 15 tests for all type expressions
- **Advanced Features**: 56 tests (attributes, generics, patterns, macros)
- **Error Recovery**: 23 tests for comprehensive error handling
- **Control Flow Integration**: 23 tests for complete lexer-parser integration
- **New Coverage Enhancements**: 27 tests (array indexing, stress testing, performance)

**Recent Test Additions:**
- **Array Indexing Coverage**: 12 comprehensive tests for array operations
- **Parser Stress Coverage**: 15 tests for boundary conditions and performance
- **String Literal Fixes**: Resolved all string parsing issues with proper parser context

**Quality Metrics:**
- **100% Pass Rate**: All 429 tests passing consistently
- **Zero Warnings**: Clean code with no clippy warnings
- **Performance Validated**: Stress tests ensure scalability
- **Production Ready**: Comprehensive coverage of real-world scenarios

---

## Getting Started

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required tools
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-fuzz       # Fuzz testing
cargo install criterion        # Benchmarking
cargo install cargo-audit      # Security auditing

# Clone repository
git clone https://github.com/ferra-lang/ferra-lang.git
cd ferra-lang/crates/ferra_parser
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
cargo bench
```

---

## Architecture Overview

### Core Components

```
ferra_parser/
├── src/
│   ├── ast/           # AST node definitions and traits
│   ├── error/         # Error types and recovery mechanisms
│   ├── pratt/         # Pratt parser for expressions
│   ├── program/       # Program-level parsing
│   ├── statement/     # Statement parsing
│   ├── token/         # Token stream abstraction
│   ├── types/         # Type annotation parsing
│   ├── block/         # Block structure parsing
│   ├── pattern/       # Pattern parsing for match expressions
│   ├── attribute/     # Attribute parsing (#[derive] syntax)
│   ├── generic/       # Generic type parameter parsing
│   ├── macro_parser/  # Macro system foundation
│   └── lib.rs         # Public API surface
├── tests/             # Integration tests (336 tests)
├── benches/           # Performance benchmarks
├── examples/          # Usage examples
└── docs/              # Documentation files
```

### Key Abstractions

#### 1. Arena Allocation System
- **Purpose**: Zero-copy AST with automatic memory management
- **Implementation**: Uses `bumpalo` for efficient allocation
- **Pattern**: All AST nodes are allocated in a single arena

```rust
// AST nodes are arena-allocated
pub struct Expression {/* arena-allocated fields */}

// Parser takes arena reference
impl<'arena> Parser<'arena> {
    pub fn new(arena: &'arena Arena) -> Self { }
}
```

#### 2. Token Stream Abstraction
- **Purpose**: Flexible input handling for different token sources
- **Trait**: `TokenStream` for peek/consume operations
- **Implementations**: `VecTokenStream`, custom streams

```rust
pub trait TokenStream {
    fn peek(&self) -> &Token;
    fn consume(&mut self) -> Token;
    fn is_at_end(&self) -> bool;
}
```

#### 3. Pratt Expression Parser
- **Purpose**: Proper operator precedence and associativity
- **Pattern**: NUD (Null Denotation) and LED (Left Denotation) handlers
- **Features**: Configurable precedence, error recovery

#### 4. Error Recovery System
- **Purpose**: Continue parsing after syntax errors
- **Strategy**: Panic mode recovery to synchronization points
- **Goal**: Positive-first error messaging

---

## Development Workflow

### Feature Development Process

1. **Create Feature Branch**
```bash
git checkout -b feat/your-feature-name
```

2. **Write Tests First** (TDD approach)
```bash
# Add test case to appropriate test file
# tests/test_your_feature.rs

#[test]
fn test_your_feature_basic() {
    let source = "your test code";
    let ast = parse_program(source);
    // Assertions
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
cargo bench  # Run benchmarks
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
git commit -m "feat: add support for async function parsing"

# Bug fixes
git commit -m "fix: resolve infinite loop in error recovery"

# Performance improvements
git commit -m "perf: optimize expression parsing with fast paths"

# Documentation
git commit -m "docs: update parser API documentation"

# Tests
git commit -m "test: add comprehensive async function test coverage"

# Refactoring
git commit -m "refactor: simplify error recovery mechanism"
```

---

## Parser-Specific Development Guidelines

### Adding New Language Features

When adding new syntax to the parser:

#### Step 1: Grammar Design
```rust
// Document the grammar production in comments
/// new_feature ::= KEYWORD identifier '(' parameter_list ')' block
///
/// Examples:
/// - `keyword my_feature(param1, param2) { body }`
/// - `keyword simple() { statement; }`
```

#### Step 2: AST Node Addition
```rust
// In src/ast/nodes.rs
#[derive(Debug, Clone)]
pub struct NewFeature<'arena> {
    pub keyword_span: SourceSpan,
    pub name: &'arena str,
    pub parameters: &'arena [Parameter<'arena>],
    pub body: Block<'arena>,
}
```

#### Step 3: Parser Implementation
```rust
// In appropriate parser module
impl<'arena, S: TokenStream> MyParser<'arena, S> {
    pub fn parse_new_feature(&mut self) -> Result<NewFeature<'arena>, ParseError> {
        // Expect keyword
        self.expect_token(TokenType::NewKeyword)?;
        
        // Parse identifier
        let name = self.parse_identifier()?;
        
        // Parse parameter list
        self.expect_token(TokenType::LeftParen)?;
        let parameters = self.parse_parameter_list()?;
        self.expect_token(TokenType::RightParen)?;
        
        // Parse body
        let body = self.parse_block()?;
        
        Ok(NewFeature {
            keyword_span: self.current_span(),
            name,
            parameters,
            body,
        })
    }
}
```

#### Step 4: Comprehensive Testing
```rust
// In tests/test_new_feature.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_new_feature() {
        let source = "keyword simple() { let x = 42; }";
        let arena = Arena::new();
        let tokens = tokenize(source).unwrap();
        let mut parser = MyParser::new(&arena, tokens);
        
        let result = parser.parse_new_feature().unwrap();
        assert_eq!(result.name, "simple");
        assert_eq!(result.parameters.len(), 0);
        assert_eq!(result.body.statements.len(), 1);
    }
    
    #[test]
    fn test_new_feature_with_parameters() {
        let source = "keyword complex(x: i32, y: String) { process(x, y); }";
        // ... comprehensive test
    }
    
    #[test]
    fn test_new_feature_error_cases() {
        // Missing name
        assert_parse_error("keyword () { }");
        
        // Missing body
        assert_parse_error("keyword test(x: i32)");
        
        // Invalid parameter syntax
        assert_parse_error("keyword test(x,) { }");
    }
}
```

### AST Node Development

#### Adding New AST Nodes

1. **Define Node Structure**
```rust
// In src/ast/nodes.rs
#[derive(Debug, Clone, PartialEq)]
pub struct YourNewNode {
    pub field1: String,
    pub field2: Option<&'arena Expression>,
    pub span: Span,
}
```

2. **Add to Expression/Statement Enum**
```rust
// In appropriate enum
pub enum Expression {
    // existing variants...
    YourNewNode(YourNewNode),
}
```

3. **Implement Parsing Logic**
```rust
// In relevant parser module
impl<'arena> Parser<'arena> {
    fn parse_your_new_node(&mut self) -> Result<YourNewNode, ParseError> {
        // Implementation
    }
}
```

4. **Add Test Coverage**
```rust
#[test]
fn test_your_new_node_parsing() {
    let source = "your syntax here";
    let ast = parse_program(source);
    // Verify AST structure
}
```

5. **Update Visitor Pattern** (if applicable)
```rust
// In src/ast/visitor.rs
pub trait AstVisitor {
    fn visit_your_new_node(&mut self, node: &YourNewNode) -> VisitResult {
        VisitResult::Continue
    }
}
```

### Expression Parser Development

#### Adding New Operators

1. **Define Operator**
```rust
// In src/ast/operators.rs
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // existing operators...
    YourNewOperator,
}
```

2. **Add Precedence**
```rust
// In src/pratt/precedence.rs
impl BinaryOperator {
    pub fn binding_power(&self) -> (BindingPower, BindingPower) {
        match self {
            // existing cases...
            YourNewOperator => (50, 51), // left binding, right binding
        }
    }
}
```

3. **Implement LED Handler**
```rust
// In src/pratt/parser.rs
fn handle_led(&mut self, left: &'arena Expression, token: &Token) -> Result<&'arena Expression, ParseError> {
    match &token.token_type {
        TokenType::YourNewOperatorToken => {
            self.parse_binary_expression(left, BinaryOperator::YourNewOperator, token)
        }
        // existing cases...
    }
}
```

4. **Add Comprehensive Tests**
```rust
#[test]
fn test_your_new_operator_precedence() {
    test_expr!("a your_op b + c" => /* expected precedence structure */);
}

#[test]
fn test_your_new_operator_associativity() {
    test_expr!("a your_op b your_op c" => /* expected associativity */);
}
```

### Error Handling Development

#### Adding New Error Types

1. **Define Error Variant**
```rust
// In src/error/mod.rs
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParseError {
    // existing variants...
    
    #[error("Your error description: {message}")]
    YourNewError {
        message: String,
        span: Span,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
```

2. **Implement Error Construction**
```rust
impl ParseError {
    pub fn your_new_error(message: String, span: Span) -> Self {
        Self::YourNewError {
            message,
            span,
            source: None,
        }
    }
}
```

3. **Add Recovery Strategy**
```rust
// In src/error/recovery.rs
impl ErrorRecovery {
    pub fn handle_your_new_error(&mut self, error: &ParseError) -> RecoveryAction {
        match error {
            ParseError::YourNewError { .. } => {
                // Define recovery strategy
                RecoveryAction::SkipToNextStatement
            }
            // existing cases...
        }
    }
}
```

4. **Add to Error Catalog**
```rust
// Update ERROR_CATALOG.md with new error documentation
```

### Improving Error Messages

Follow positive-first messaging principles:

```rust
// ❌ Don't: Blame-focused messages
return Err(ParseError::new("Invalid syntax"));
return Err(ParseError::new("Parse failed"));

// ✅ Do: Constructive messages
return Err(ParseError::expected_token(
    TokenType::Identifier, 
    found_token,
    "Function names must be valid identifiers"
));

return Err(ParseError::missing_element(
    "function body",
    current_span,
    "Add { statements } after parameter list"
));
```

#### Error Message Template
```rust
#[derive(Error, Diagnostic, Debug)]
#[error("Expected {expected}, found {found}")]
#[diagnostic(
    code(ferra::parser::E042),
    help("Try {suggestion}")
)]
pub struct SpecificError {
    pub expected: String,
    pub found: String,
    pub suggestion: String,
    #[label("Expected {expected} here")]
    pub span: SourceSpan,
}
```

---

## Testing Guidelines

### Test Structure

#### Unit Tests (in `src/`)
- **Purpose**: Test individual components in isolation
- **Location**: `#[cfg(test)] mod tests` in each module
- **Focus**: Parser components, AST nodes, error handling

#### Integration Tests (in `tests/`)
- **Purpose**: Test complete parsing scenarios
- **Organization**: One file per feature area
- **Coverage**: All language features end-to-end

#### Benchmarks (in `benches/`)
- **Purpose**: Performance regression detection
- **Categories**: Parser creation, expression parsing, memory usage
- **Baseline**: Establish performance expectations

### Writing Effective Tests

#### Test Naming Convention
```rust
// Unit tests: test_{component}_{scenario}
#[test]
fn test_pratt_parser_precedence_handling() { }

// Integration tests: test_{feature}_{scenario}
#[test]
fn test_async_functions_with_complex_signatures() { }

// Benchmarks: {category}_{specific_case}
fn expression_parsing_deeply_nested(c: &mut Criterion) { }
```

#### Test Patterns

**Expression Testing Pattern:**
```rust
#[test]
fn test_new_expression_type() {
    let arena = test_arena();
    let source = "your expression syntax";
    let mut parser = test_parser(&arena, source);
    
    let expr = parser.parse_expression().unwrap();
    
    match expr {
        Expression::YourNewType(node) => {
            assert_eq!(node.field, expected_value);
            assert_eq!(node.span, expected_span);
        }
        _ => panic!("Expected YourNewType expression"),
    }
}
```

**Error Testing Pattern:**
```rust
#[test]
fn test_error_scenario() {
    let arena = test_arena();
    let source = "invalid syntax here";
    let mut parser = test_parser(&arena, source);
    
    match parser.parse_expression() {
        Err(ParseError::YourExpectedError { message, .. }) => {
            assert!(message.contains("expected text"));
        }
        other => panic!("Expected YourExpectedError, got {:?}", other),
    }
}
```

**Recovery Testing Pattern:**
```rust
#[test]
fn test_error_recovery() {
    let source = r#"
        fn broken( {  // Syntax error
            let x = 5;
        }
        
        fn good() -> i32 {  // Should still parse
            return 42;
        }
    "#;
    
    let arena = Arena::new();
    let tokens = ferra_lexer::tokenize(source).unwrap();
    let mut parser = Parser::new(&arena, tokens);
    
    match parser.parse_compilation_unit() {
        Err(errors) => {
            assert!(errors.len() > 0);
            // Verify partial AST contains recovered items
            assert!(parser.partial_result().items.len() > 0);
        }
        Ok(_) => panic!("Expected parse errors"),
    }
}
```

### Performance Testing

#### Adding Benchmarks

```rust
// In benches/your_feature_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn your_feature_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("your_feature");
    
    for size in &[10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("complexity", size),
            size,
            |b, &size| {
                let source = generate_test_input(size);
                let arena = Arena::new();
                let tokens = ferra_lexer::tokenize(&source).unwrap();
                
                b.iter(|| {
                    let mut parser = Parser::new(&arena, tokens.clone());
                    parser.parse_your_feature().unwrap()
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, your_feature_benchmark);
criterion_main!(benches);
```

#### Memory Testing

```rust
#[test]
fn test_memory_usage() {
    let sizes = [100, 500, 1000, 2000];
    let mut memory_per_item = Vec::new();
    
    for size in sizes {
        let source = generate_program_with_items(size);
        
        let (ast, tracker) = MemoryTracker::track_parsing(|| {
            parse_program(&source)
        });
        
        let memory_per_item_ratio = tracker.memory_growth() / size;
        memory_per_item.push(memory_per_item_ratio);
    }
    
    // Memory usage should be roughly linear
    for i in 1..memory_per_item.len() {
        let ratio = memory_per_item[i] as f64 / memory_per_item[0] as f64;
        assert!(ratio < 2.0, "Memory usage is not scaling linearly");
    }
}
```

---

## Code Quality Standards

### Code Style

#### Formatting
```bash
# Use rustfmt with default settings
cargo fmt

# Check formatting in CI
cargo fmt --check
```

#### Linting
```bash
# Pass all clippy lints
cargo clippy --all-targets -- -D warnings

# Address specific clippy suggestions
cargo clippy --fix
```

#### Documentation
```rust
/// Brief description of what the function does
/// 
/// # Arguments
/// 
/// * `param1` - Description of parameter
/// * `param2` - Description of parameter
/// 
/// # Returns
/// 
/// Description of return value
/// 
/// # Errors
/// 
/// Description of when this function returns an error
/// 
/// # Examples
/// 
/// ```rust
/// use ferra_parser::Parser;
/// 
/// let result = function_example(param1, param2)?;
/// assert_eq!(result.field, expected_value);
/// ```
pub fn well_documented_function(param1: Type1, param2: Type2) -> Result<ReturnType, ErrorType> {
    // Implementation
}
```

### Error Handling

#### Parser Error Guidelines

1. **Use Structured Errors**: Always use the `ParseError` enum
2. **Provide Context**: Include spans and helpful messages
3. **Positive Language**: Focus on what should be done
4. **Recovery**: Always attempt error recovery when possible

```rust
// Good error handling
fn parse_function_declaration(&mut self) -> Result<FunctionDecl, ParseError> {
    let start_span = self.current_span();
    
    // Expect 'fn' keyword
    if !self.expect_token(TokenType::Fn) {
        return Err(ParseError::unexpected_token(
            "function declaration".to_string(),
            self.current_token().clone(),
            "Expected 'fn' keyword to start function declaration".to_string(),
        ));
    }
    
    // Parse function name
    let name = match self.parse_identifier() {
        Ok(name) => name,
        Err(error) => {
            // Attempt recovery
            self.recover_to_next_item();
            return Err(error);
        }
    };
    
    // Continue parsing...
}
```

### Performance Guidelines

#### Allocation Patterns
```rust
// Good: Use arena for AST nodes
let expr = self.arena.alloc(Expression::Binary(BinaryExpression {
    left: left_expr,
    operator: BinaryOperator::Add,
    right: right_expr,
    span: combined_span,
}));

// Good: Pre-allocate vectors when size is known
let mut parameters = Vec::with_capacity(expected_param_count);

// Avoid: Unnecessary cloning
let token = self.tokens.peek().clone(); // Only if necessary
```

#### Parser State Management
```rust
// Good: Minimize parser state
struct Parser<'arena, T: TokenStream> {
    arena: &'arena Arena,
    tokens: T,
    // Minimal additional state
}

// Good: Use local variables for temporary state
fn parse_complex_construct(&mut self) -> Result<ConstructType, ParseError> {
    let mut local_state = ConstructState::new();
    // Use local_state instead of parser fields
}
```

### Performance Optimization

When optimizing parser performance:

#### Benchmark First
```rust
// In benches/parser_bench.rs
fn bench_new_feature_parsing(c: &mut Criterion) {
    let source = generate_test_input(1000); // Generate large input
    
    c.bench_function("new_feature_parsing", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let tokens = tokenize(black_box(&source)).unwrap();
            let mut parser = MyParser::new(&arena, tokens);
            black_box(parser.parse_new_feature().unwrap());
        });
    });
}
```

#### Common Optimization Patterns
```rust
// ✅ Good: Minimize allocations
fn parse_list<T>(&mut self, parse_item: impl Fn(&mut Self) -> Result<T, ParseError>) 
    -> Result<&'arena [T], ParseError> 
{
    let mut items = Vec::new();
    // ... parse items
    Ok(self.arena.alloc_slice(&items))
}

// ✅ Good: Early returns for common cases
fn parse_simple_or_complex(&mut self) -> Result<Expression<'arena>, ParseError> {
    if self.peek().is_simple_literal() {
        return self.parse_simple_literal(); // Fast path
    }
    self.parse_complex_expression() // Slower path
}

// ✅ Good: Reuse token stream efficiently
fn parse_multiple_items(&mut self) -> Result<Items<'arena>, ParseError> {
    // Avoid unnecessary peek() calls in loops
    while !matches!(self.current_token(), TokenType::Eof | TokenType::RightBrace) {
        let item = self.parse_item()?;
        items.push(item);
    }
}
```

---

## Common Patterns

### Parser Combinator Pattern

```rust
// Reusable parsing patterns
impl<'arena, T: TokenStream> Parser<'arena, T> {
    /// Parse optional construct with default
    fn parse_optional<F, R>(&mut self, parser_fn: F, default: R) -> R
    where
        F: FnOnce(&mut Self) -> Result<R, ParseError>,
    {
        match parser_fn(self) {
            Ok(result) => result,
            Err(_) => default,
        }
    }
    
    /// Parse comma-separated list
    fn parse_comma_separated<F, R>(&mut self, parser_fn: F) -> Result<Vec<R>, ParseError>
    where
        F: Fn(&mut Self) -> Result<R, ParseError>,
    {
        let mut items = Vec::new();
        
        loop {
            items.push(parser_fn(self)?);
            
            if !self.consume_if_token(TokenType::Comma) {
                break;
            }
        }
        
        Ok(items)
    }
    
    /// Parse delimited construct
    fn parse_delimited<F, R>(
        &mut self,
        open: TokenType,
        close: TokenType,
        parser_fn: F,
    ) -> Result<R, ParseError>
    where
        F: FnOnce(&mut Self) -> Result<R, ParseError>,
    {
        self.expect_token(open)?;
        let result = parser_fn(self)?;
        self.expect_token(close)?;
        Ok(result)
    }
}
```

### Error Recovery Pattern

```rust
impl<'arena, T: TokenStream> Parser<'arena, T> {
    /// Generic error recovery to synchronization points
    fn recover_to_sync_point(&mut self, sync_tokens: &[TokenType]) -> bool {
        while !self.is_at_end() {
            let current_token = self.peek_token().token_type;
            
            if sync_tokens.contains(&current_token) {
                return true;
            }
            
            self.consume_token();
        }
        
        false
    }
    
    /// Recover to next statement
    fn recover_to_statement(&mut self) -> bool {
        self.recover_to_sync_point(&[
            TokenType::Semicolon,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Fn,
            TokenType::Let,
            TokenType::Var,
        ])
    }
}
```

---

## Debugging Guidelines

### Parser Debugging

#### Debug Logging
```rust
// Use debug logging for parser state
#[cfg(debug_assertions)]
macro_rules! debug_parse {
    ($($arg:tt)*) => {
        eprintln!("[PARSER DEBUG] {}", format!($($arg)*));
    };
}

fn parse_complex_expression(&mut self) -> Result<Expression, ParseError> {
    debug_parse!("Parsing expression at token: {:?}", self.peek_token());
    
    let result = self.parse_expression_impl();
    
    debug_parse!("Expression result: {:?}", result);
    result
}
```

#### Test Debugging
```rust
#[test]
fn debug_specific_parsing_issue() {
    let source = "problematic code here";
    
    // Enable debug output
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    let arena = Arena::new();
    let tokens = ferra_lexer::tokenize(source).unwrap();
    
    // Print tokens for debugging
    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
    
    let mut parser = Parser::new(&arena, VecTokenStream::new(tokens));
    let result = parser.parse_compilation_unit();
    
    match result {
        Ok(ast) => {
            println!("Successfully parsed AST: {:#?}", ast);
        }
        Err(errors) => {
            for error in errors {
                println!("Parse error: {:#?}", error);
            }
        }
    }
}
```

### Performance Debugging

```rust
#[cfg(test)]
mod performance_debug {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn profile_parsing_performance() {
        let source = generate_large_program();
        
        let start = Instant::now();
        let tokens = ferra_lexer::tokenize(&source).unwrap();
        let tokenization_time = start.elapsed();
        
        let arena = Arena::new();
        let start = Instant::now();
        let mut parser = Parser::new(&arena, VecTokenStream::new(tokens));
        let parser_creation_time = start.elapsed();
        
        let start = Instant::now();
        let result = parser.parse_compilation_unit();
        let parsing_time = start.elapsed();
        
        println!("Tokenization: {:?}", tokenization_time);
        println!("Parser creation: {:?}", parser_creation_time);
        println!("Parsing: {:?}", parsing_time);
        
        if let Ok(ast) = result {
            println!("AST nodes: {}", ast.node_count());
            println!("Time per node: {:?}", parsing_time / ast.node_count() as u32);
        }
    }
}
```

---

## Common Development Issues

### Debugging and Troubleshooting

#### 1. "Parser hanging in infinite loop"
```rust
// Debug by adding trace logging
fn parse_problematic_construct(&mut self) -> Result<Node, ParseError> {
    log::trace!("Entering parse_problematic_construct at {:?}", self.current_token());
    
    // Ensure progress is made
    let start_position = self.position();
    
    // ... parsing logic
    
    if self.position() == start_position {
        return Err(ParseError::no_progress("parse_problematic_construct"));
    }
    
    log::trace!("Exiting parse_problematic_construct successfully");
    Ok(node)
}
```

#### 2. "AST lifetime issues"
```rust
// ❌ Problem: Trying to return AST that outlives arena
fn parse_and_extract<'a>(arena: &'a Arena, source: &str) -> CompilationUnit<'a> {
    // This won't compile - arena reference is temporary
}

// ✅ Solution: Extract data before returning
fn parse_and_extract(source: &str) -> ExtractedData {
    let arena = Arena::new();
    let ast = parse_with_arena(&arena, source)?;
    extract_data_from_ast(ast) // Extract before arena drops
}
```

#### 3. "Test failures in CI but not locally"
```rust
// Ensure tests are deterministic
#[test]
fn test_feature() {
    // ❌ Don't rely on HashMap iteration order
    let mut map = HashMap::new();
    
    // ✅ Use BTreeMap or Vec with sort
    let mut items: Vec<_> = map.into_iter().collect();
    items.sort_by_key(|&(ref k, _)| k);
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

2. **Code Quality**
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo audit
```

3. **Documentation**
```bash
cargo doc --no-deps --open
# Review generated documentation
```

4. **Performance Baseline**
```bash
cargo bench > baseline.txt
# Compare with previous baseline
```

5. **Update Version Numbers**
```toml
# Cargo.toml
[package]
version = "1.1.0"  # Update version
```

### Release Documentation

1. **Update CHANGELOG.md**
```markdown
## [1.1.0] - 2025-01-XX

### Added
- New feature descriptions

### Changed
- Modified functionality descriptions

### Fixed
- Bug fix descriptions

### Performance
- Performance improvement descriptions
```

2. **Update Documentation**
- API documentation
- Usage examples
- Migration guide (if needed)

---

## Getting Help

### Resources
- **Design Documentation**: `DESIGN_IMPLEMENTATION_PLAN.md`
- **Test Documentation**: `TEST_INFRASTRUCTURE.md`
- **Error Catalog**: `ERROR_CATALOG.md`
- **User API Guide**: `USER_API_GUIDE.md`

### Community
- **Discussions**: GitHub Discussions for questions
- **Issues**: GitHub Issues for bugs and feature requests
- **Pull Requests**: Code review and collaboration

### Contact
- **Maintainers**: See MAINTAINERS.md
- **Security Issues**: security@ferra-lang.org

### Mentorship Program

New contributors can request mentorship for:
- Understanding parser architecture
- Learning about recursive descent parsing
- Getting familiar with Rust arena allocation
- Contributing to specific features

---

**Ready to contribute?** Start by exploring [good first issues](https://github.com/your-org/ferra-lang/labels/good%20first%20issue) or reading through the [USER_API_GUIDE.md](./USER_API_GUIDE.md) to understand how the parser works. 