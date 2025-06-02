# Ferra Parser Test Infrastructure

**Version**: 1.0  
**Last Updated**: January 2025  
**Phase 2 Complete**: All testing frameworks and utilities documented  

---

## Overview

The Ferra Parser test infrastructure provides comprehensive testing capabilities with 429 tests covering all language features. The testing framework includes unit tests, integration tests, benchmarks, fuzzing, and specialized testing utilities.

### Test Categories

| Type              | Count | Coverage                | Purpose                       |
|-------------------|-------|-------------------------|-------------------------------|
| **Unit Tests**    | 63    | Core library components | Component isolation testing   |
| **Integration Tests** | 336   | Full language features  | End-to-end parsing validation |
| **Benchmarks**    | 15    | Performance regression  | Performance monitoring        |
| **Fuzz Tests**    | 5     | Edge case discovery     | Robustness validation         |
| **Stress Tests**  | 25    | Large-scale scenarios   | Scalability verification      |

---

## Test Organization

### Directory Structure

```
crates/ferra_parser/
├── src/                           # Unit tests (63 tests)
│   ├── ast/mod.rs                 # AST node tests (3)
│   ├── token/mod.rs               # Token stream tests (3)
│   ├── pratt/mod.rs               # Pratt parser tests (5)
│   ├── error/mod.rs               # Error handling tests (6)
│   ├── types/mod.rs               # Type system tests (15)
│   ├── program/mod.rs             # Program parsing tests (31)
│   ├── block/mod.rs               # Block parsing tests
│   ├── statement/mod.rs           # Statement parsing tests
│   ├── pattern/mod.rs             # Pattern parsing tests
│   ├── attribute/mod.rs           # Attribute parsing tests
│   ├── generic/mod.rs             # Generic type parameter tests
│   └── macro_parser/mod.rs        # Macro system tests
├── tests/                         # Integration tests (336 tests)
│   ├── test_expressions.rs        # Expression parsing (27)
│   ├── test_control_flow_integration.rs  # Control flow (23)
│   ├── test_async_functions.rs    # Async features (9)
│   ├── test_modifier_combinations.rs     # Modifiers (18)
│   ├── test_error_recovery_stress.rs     # Error recovery (6)
│   ├── test_memory_profiling.rs   # Memory testing (8)
│   └── [18 more test files]       # Additional features
├── benches/                       # Performance benchmarks
│   ├── parser_benchmarks.rs       # Core benchmarks
│   └── memory_benchmarks.rs       # Memory profiling (Note: this file needs to be created)
└── examples/                      # Usage examples (directory exists, currently empty)
    # Examples would include:
    # ├── basic_parsing.rs
    # ├── error_handling.rs
    # └── performance_testing.rs
```

### Test Naming Conventions

```rust
// Unit test naming: test_{component}_{aspect}
#[test]
fn test_arena_allocation() { }
#[test]
fn test_pratt_parser_precedence() { }
#[test]
fn test_error_recovery_sync_tokens() { }

// Integration test naming: test_{feature}_{scenario}
#[test]
fn test_function_declaration_basic() { }
#[test]
fn test_control_flow_nested_loops() { }
#[test]
fn test_async_function_with_modifiers() { }

// Benchmark naming: {category}_{specific_case}
fn parser_creation(c: &mut Criterion) { }
fn expression_parsing_complex(c: &mut Criterion) { }
fn memory_allocation_scaling(c: &mut Criterion) { }
```

---

## Unit Testing Framework

### Test Utilities (`src/test_utils.rs`)

Core testing utilities for parser components:

```rust
/// Create test arena for unit tests
pub fn test_arena() -> Arena {
    Arena::new()
}

/// Create mock token stream from source
pub fn mock_tokens(source: &str) -> VecTokenStream {
    let tokens = ferra_lexer::tokenize(source).unwrap();
    VecTokenStream::new(tokens)
}

/// Create parser for testing
pub fn test_parser(arena: &Arena, source: &str) -> Parser<VecTokenStream> {
    let tokens = mock_tokens(source);
    Parser::new(arena, tokens)
}

/// Assert AST node types
pub fn assert_expression_type(expr: &Expression, expected: ExpressionType) {
    match (expr, expected) {
        (Expression::Binary(_), ExpressionType::Binary) => {},
        (Expression::FunctionCall(_), ExpressionType::FunctionCall) => {},
        _ => panic!("Expected {:?}, found {:?}", expected, expr),
    }
}

/// Create test spans for testing
pub fn test_span(start: usize, end: usize) -> Span {
    Span::new(start, end, "test.ferra")
}
```

### Test Macros

Specialized macros for common test patterns:

```rust
/// Test expression parsing with expected result
macro_rules! test_expr {
    ($source:expr => $expected:pat) => {
        let arena = test_arena();
        let mut parser = test_parser(&arena, $source);
        let expr = parser.parse_expression().unwrap();
        assert!(matches!(expr, $expected));
    };
}

/// Test parsing error with expected error type
macro_rules! test_parse_error {
    ($source:expr => $error_type:path) => {
        let arena = test_arena();
        let mut parser = test_parser(&arena, $source);
        let result = parser.parse_expression();
        assert!(matches!(result, Err($error_type { .. })));
    };
}

/// Test statement parsing
macro_rules! test_stmt {
    ($source:expr => $expected:pat) => {
        let arena = test_arena();
        let mut parser = test_parser(&arena, $source);
        let stmt = parser.parse_statement().unwrap();
        assert!(matches!(stmt, $expected));
    };
}

// Usage examples
#[test]
fn test_binary_expressions() {
    test_expr!("1 + 2" => Expression::Binary(_));
    test_expr!("x * y" => Expression::Binary(_));
    test_expr!("a && b" => Expression::Binary(_));
}

#[test]
fn test_malformed_expressions() {
    test_parse_error!("+ 5" => ParseError::UnexpectedToken);
    test_parse_error!("1 +" => ParseError::UnexpectedEof);
}
```

### Component Testing Patterns

#### AST Node Testing

```rust
#[cfg(test)]
mod ast_tests {
    use super::*;
    
    #[test]
    fn test_arena_allocation() {
        let arena = Arena::new();
        
        // Test basic allocation
        let expr = arena.alloc(Expression::IntegerLiteral(IntegerLiteral {
            value: 42,
            span: test_span(0, 2),
        }));
        
        assert_eq!(arena.node_count(), 1);
        assert!(matches!(expr, Expression::IntegerLiteral(_)));
    }
    
    #[test]
    fn test_arena_slice_allocation() {
        let arena = Arena::new();
        let items = vec![1, 2, 3, 4, 5];
        let arena_slice = arena.alloc_slice(&items);
        
        assert_eq!(arena_slice.len(), 5);
        assert_eq!(arena_slice[2], 3);
    }
    
    #[test]
    fn test_arena_reset() {
        let arena = Arena::new();
        arena.alloc(42i32);
        arena.alloc(84i32);
        
        assert_eq!(arena.node_count(), 2);
        
        arena.reset();
        assert_eq!(arena.node_count(), 0);
    }
}
```

#### Parser Component Testing

```rust
#[cfg(test)]
mod pratt_tests {
    use super::*;
    
    #[test]
    fn test_precedence_binding() {
        let arena = test_arena();
        let mut parser = test_parser(&arena, "1 + 2 * 3");
        let expr = parser.parse_expression().unwrap();
        
        if let Expression::Binary(bin) = expr {
            // Should be: (1) + (2 * 3)
            assert!(matches!(bin.left, Expression::IntegerLiteral(_)));
            assert!(matches!(bin.right, Expression::Binary(_)));
            assert_eq!(bin.operator, BinaryOperator::Add);
        } else {
            panic!("Expected binary expression");
        }
    }
    
    #[test]
    fn test_associativity() {
        let arena = test_arena();
        let mut parser = test_parser(&arena, "a = b = c");
        let expr = parser.parse_expression().unwrap();
        
        if let Expression::Binary(bin) = expr {
            // Right associative: a = (b = c)
            assert!(matches!(bin.left, Expression::Identifier(_)));
            assert!(matches!(bin.right, Expression::Binary(_)));
        }
    }
}
```

#### Error Handling Testing

```rust
#[cfg(test)]
mod error_tests {
    use super::*;
    
    #[test]
    fn test_error_recovery() {
        let arena = test_arena();
        let source = "fn broken( { let x = 5; } fn good() -> i32 { return 42; }";
        let mut parser = test_parser(&arena, source);
        
        match parser.parse_compilation_unit() {
            Ok(_) => panic!("Expected parse errors"),
            Err(errors) => {
                assert!(errors.len() > 0);
                // Check that recovery allowed parsing the second function
                let partial_ast = parser.partial_result();
                assert!(partial_ast.items.len() > 0);
            }
        }
    }
    
    #[test]
    fn test_error_message_quality() {
        let arena = test_arena();
        let mut parser = test_parser(&arena, "let x = ;");
        
        match parser.parse_statement() {
            Err(ParseError::UnexpectedToken { expected, found, .. }) => {
                assert!(expected.contains("expression"));
                assert_eq!(found.token_type, TokenType::Semicolon);
            }
            _ => panic!("Expected unexpected token error"),
        }
    }
}
```

---

## Integration Testing Framework

### Test Organization

Integration tests are organized by feature area with consistent naming:

```rust
// test_expressions.rs - Expression parsing tests
#[test]
fn test_literal_expressions() { /* Basic literals */ }
#[test]
fn test_binary_expressions() { /* Binary operators */ }
#[test]
fn test_complex_expressions() { /* Nested expressions */ }

// test_control_flow_integration.rs - Control flow tests
#[test]
fn test_if_statements() { /* If/else parsing */ }
#[test]
fn test_loop_statements() { /* While/for loops */ }
#[test]
fn test_nested_control_flow() { /* Complex nesting */ }

// test_modifier_combinations.rs - Modifier tests
#[test]
fn test_function_modifiers() { /* pub, unsafe, async */ }
#[test]
fn test_variable_modifiers() { /* pub let/var */ }
#[test]
fn test_invalid_combinations() { /* Error cases */ }
```

### Integration Test Utilities (`tests/common/mod.rs`)

Shared utilities for integration tests:

```rust
/// Parse complete program and expect success
pub fn parse_program(source: &str) -> CompilationUnit {
    let arena = Arena::new();
    let tokens = ferra_lexer::tokenize(source).unwrap();
    let mut parser = Parser::new(&arena, tokens);
    parser.parse_compilation_unit().unwrap()
}

/// Parse program and expect specific number of errors
pub fn parse_with_errors(source: &str, expected_errors: usize) -> Vec<ParseError> {
    let arena = Arena::new();
    let tokens = ferra_lexer::tokenize(source).unwrap();
    let mut parser = Parser::new(&arena, tokens);
    
    match parser.parse_compilation_unit() {
        Ok(_) => panic!("Expected {} errors, got none", expected_errors),
        Err(errors) => {
            assert_eq!(errors.len(), expected_errors, 
                      "Expected {} errors, got {}", expected_errors, errors.len());
            errors
        }
    }
}

/// Test program fixtures from files
pub fn load_test_fixture(name: &str) -> String {
    let fixture_path = format!("tests/fixtures/{}.ferra", name);
    std::fs::read_to_string(fixture_path).unwrap()
}

/// Assert AST structure matches pattern
pub fn assert_ast_structure(ast: &CompilationUnit, pattern: &AstPattern) {
    // Complex AST structure validation
    pattern.validate(ast);
}

/// Generate test programs programmatically
pub fn generate_test_program(config: &TestProgramConfig) -> String {
    TestProgramGenerator::new(config).generate()
}
```

### Test Fixtures

Organized test fixtures for complex scenarios:

```
tests/fixtures/
├── simple_programs/
│   ├── hello_world.ferra
│   ├── calculator.ferra
│   └── data_structures.ferra
├── complex_programs/
│   ├── async_server.ferra
│   ├── generic_collections.ferra
│   └── error_handling.ferra
├── malformed_programs/
│   ├── syntax_errors.ferra
│   ├── incomplete_constructs.ferra
│   └── mixed_errors.ferra
└── performance_programs/
    ├── large_function_set.ferra
    ├── deep_nesting.ferra
    └── wide_structures.ferra
```

### Integration Test Examples

#### Feature Testing

```rust
#[test]
fn test_async_function_integration() {
    let source = r#"
        pub async fn fetch_data(url: String) -> Result<String, Error> {
            let response = await http_client.get(url);
            return response.text();
        }
        
        unsafe async fn dangerous_async() {
            // Unsafe async operations
        }
    "#;
    
    let ast = parse_program(source);
    assert_eq!(ast.items.len(), 2);
    
    // First function: pub async
    if let TopLevelItem::Function(func) = &ast.items[0] {
        assert!(func.is_pub);
        assert!(func.is_async);
        assert!(!func.is_unsafe);
        assert_eq!(func.name, "fetch_data");
    }
    
    // Second function: unsafe async
    if let TopLevelItem::Function(func) = &ast.items[1] {
        assert!(!func.is_pub);
        assert!(func.is_async);
        assert!(func.is_unsafe);
        assert_eq!(func.name, "dangerous_async");
    }
}
```

#### Error Recovery Testing

```rust
#[test]
fn test_error_recovery_stress() {
    let source = generate_test_program(&TestProgramConfig {
        function_count: 100,
        error_rate: 0.1,  // 10% of functions have errors
        error_types: vec![
            ErrorType::MissingParenthesis,
            ErrorType::InvalidModifier,
            ErrorType::MalformedExpression,
        ],
    });
    
    let errors = parse_with_errors(&source, 10);  // Expect ~10 errors
    
    // Verify error types
    let error_types: HashSet<_> = errors.iter()
        .map(|e| std::mem::discriminant(e))
        .collect();
    assert!(error_types.len() >= 2);  // Multiple error types
    
    // Verify recovery quality
    let partial_ast = parser.partial_result();
    assert!(partial_ast.items.len() >= 80);  // At least 80% recovered
}
```

---

## Performance Testing Framework

### Benchmark Organization

```rust
// benches/parser_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn parser_creation(c: &mut Criterion) {
    c.bench_function("parser_creation", |b| {
        b.iter(|| {
            let arena = Arena::new();
            let tokens = mock_simple_tokens();
            Parser::new(&arena, tokens)
        });
    });
}

fn expression_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("expression_parsing");
    
    for complexity in &["simple", "medium", "complex"] {
        group.bench_with_input(
            BenchmarkId::new("complexity", complexity),
            complexity,
            |b, &complexity| {
                let source = generate_expression(complexity);
                let arena = Arena::new();
                let tokens = ferra_lexer::tokenize(source).unwrap();
                
                b.iter(|| {
                    let mut parser = Parser::new(&arena, tokens.clone());
                    parser.parse_expression().unwrap()
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, parser_creation, expression_parsing);
criterion_main!(benches);
```

### Memory Profiling

```rust
// Memory usage tracking during parsing
pub struct MemoryTracker {
    initial_memory: usize,
    peak_memory: usize,
    final_memory: usize,
}

impl MemoryTracker {
    pub fn track_parsing<F, R>(f: F) -> (R, MemoryTracker)
    where
        F: FnOnce() -> R,
    {
        let initial = get_memory_usage();
        let result = f();
        let final_mem = get_memory_usage();
        
        (result, MemoryTracker {
            initial_memory: initial,
            peak_memory: final_mem,  // Simplified for example
            final_memory: final_mem,
        })
    }
    
    pub fn memory_growth(&self) -> usize {
        self.final_memory - self.initial_memory
    }
}

#[test]
fn test_memory_scaling() {
    for size in &[100, 500, 1000, 2000] {
        let source = generate_program_with_functions(*size);
        
        let (ast, tracker) = MemoryTracker::track_parsing(|| {
            parse_program(&source)
        });
        
        let memory_per_function = tracker.memory_growth() / size;
        
        // Memory usage should be roughly linear
        assert!(memory_per_function < 1024);  // Less than 1KB per function
        println!("Size: {}, Memory per function: {} bytes", size, memory_per_function);
    }
}
```

---

## Fuzzing Infrastructure

### Fuzz Test Setup

```rust
// fuzz/fuzz_targets/parse_expression.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use ferra_parser::{Parser, Arena};
use ferra_lexer;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        if let Ok(tokens) = ferra_lexer::tokenize(source) {
            let arena = Arena::new();
            let mut parser = Parser::new(&arena, tokens);
            
            // Should never panic, only return errors
            let _ = parser.parse_expression();
        }
    }
});

// fuzz/fuzz_targets/parse_program.rs
fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        if source.len() < 10000 {  // Limit size for performance
            if let Ok(tokens) = ferra_lexer::tokenize(source) {
                let arena = Arena::new();
                let mut parser = Parser::new(&arena, tokens);
                
                let _ = parser.parse_compilation_unit();
            }
        }
    }
});
```

### Fuzz Test Execution

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run expression fuzzing
cargo fuzz run parse_expression

# Run program fuzzing
cargo fuzz run parse_program

# Run with corpus
cargo fuzz run parse_expression -- corpus/

# Generate coverage report
cargo fuzz coverage parse_expression
```

---

## Continuous Integration Testing

### CI Test Configuration

```yaml
# .github/workflows/ci.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      # Unit and integration tests
      - name: Run tests
        run: cargo test --package ferra_parser --verbose
      
      # Documentation tests
      - name: Test documentation
        run: cargo test --doc --package ferra_parser
      
      # Benchmark smoke tests
      - name: Benchmark smoke test
        run: timeout 30s cargo bench --package ferra_parser || true
  
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      
      - name: Fuzz expression parsing
        run: timeout 300s cargo fuzz run parse_expression || true
      
      - name: Fuzz program parsing
        run: timeout 300s cargo fuzz run parse_program || true
```

### Test Quality Metrics

```rust
// Test coverage reporting
#[cfg(test)]
mod coverage_tests {
    use super::*;
    
    #[test]
    fn test_statement_coverage() {
        // Ensure all statement types are tested
        let covered_statements = get_tested_statement_types();
        let all_statements = StatementType::all_variants();
        
        for stmt_type in all_statements {
            assert!(covered_statements.contains(&stmt_type),
                   "Statement type {:?} not covered in tests", stmt_type);
        }
    }
    
    #[test]
    fn test_expression_coverage() {
        // Ensure all expression types are tested
        let covered_expressions = get_tested_expression_types();
        let all_expressions = ExpressionType::all_variants();
        
        for expr_type in all_expressions {
            assert!(covered_expressions.contains(&expr_type),
                   "Expression type {:?} not covered in tests", expr_type);
        }
    }
    
    #[test]
    fn test_error_coverage() {
        // Ensure all error types can be triggered
        let covered_errors = get_tested_error_types();
        let all_errors = ParseErrorType::all_variants();
        
        for error_type in all_errors {
            assert!(covered_errors.contains(&error_type),
                   "Error type {:?} not covered in tests", error_type);
        }
    }
}
```

---

## Test Development Guidelines

### Writing Effective Tests

1. **Test Isolation**: Each test should be independent
2. **Clear Assertions**: Use descriptive assertion messages
3. **Edge Cases**: Test boundary conditions and error cases
4. **Performance**: Include performance regression tests
5. **Documentation**: Document complex test scenarios

### Test Maintenance

```rust
// Regular test maintenance tasks
#[test]
fn test_ast_node_count() {
    // This test will fail when new AST nodes are added
    // Reminder to update test coverage
    let known_node_types = 42;  // Update when adding nodes
    let actual_node_types = AstNodeType::all_variants().len();
    
    if actual_node_types != known_node_types {
        panic!("AST node count changed from {} to {}. \
                Please update test coverage for new node types.",
                known_node_types, actual_node_types);
    }
}

// Test data freshness
#[test]
fn test_grammar_sync() {
    // Ensure test fixtures match current grammar
    let grammar_version = include_str!("../../docs/SYNTAX_GRAMMAR_V0.1.md")
        .lines()
        .find(|line| line.contains("Version:"))
        .expect("Grammar version not found");
    
    let test_grammar_version = include_str!("fixtures/grammar_version.txt")
        .trim();
    
    assert!(grammar_version.contains(test_grammar_version),
           "Test fixtures out of sync with grammar. Please update test cases.");
}
```

### Performance Test Guidelines

```rust
// Performance regression detection
#[test]
fn test_parsing_performance_regression() {
    let source = load_test_fixture("medium_complexity_program");
    
    let start = std::time::Instant::now();
    let _ast = parse_program(&source);
    let duration = start.elapsed();
    
    // Parsing should complete within reasonable time
    assert!(duration < std::time::Duration::from_millis(100),
           "Parsing took {:?}, which exceeds 100ms threshold. \
            Possible performance regression.", duration);
}

// Memory usage regression detection
#[test]
fn test_memory_usage_regression() {
    let source = load_test_fixture("large_program");
    
    let (ast, tracker) = MemoryTracker::track_parsing(|| {
        parse_program(&source)
    });
    
    let memory_per_node = tracker.memory_growth() / ast.node_count();
    
    // Memory per node should be reasonable
    assert!(memory_per_node < 256,
           "Memory usage per node is {} bytes, exceeding 256 byte threshold. \
            Possible memory regression.", memory_per_node);
}
```

This comprehensive test infrastructure ensures the Ferra parser maintains high quality, performance, and reliability across all development phases. 

## Current Test Status: 429 Parser Tests Passing

**Test Breakdown:**
- **Unit Tests**: 63 tests (within `src/` modules)
- **Integration Tests**: 336 tests (across 20+ test files)
- **Total Success Rate**: 100% (429/429 passing)

**Recent Enhancements:**
- **Array Indexing Coverage**: 12 new comprehensive tests
- **Parser Stress Coverage**: 15 new boundary and performance tests  
- **String Literal Fixes**: Resolved parsing context issues
- **Performance Regression Tests**: 2 new timing validation tests 