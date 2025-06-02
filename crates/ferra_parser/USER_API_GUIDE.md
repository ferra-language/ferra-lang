# Ferra Parser User API Guide

**Version**: 1.0  
**Last Updated**: January 2025  
**Phase 2 Complete**: All public APIs documented and stable  

---

## Overview

The Ferra Parser is a high-performance, arena-allocated recursive descent parser with Pratt expression parsing. It converts Ferra source code into a well-typed Abstract Syntax Tree (AST) with comprehensive error recovery and positive-first error messaging.

### Key Features
- **Arena Allocation**: Zero-copy AST nodes with automatic memory management
- **Pratt Expression Parsing**: Proper operator precedence and associativity
- **Error Recovery**: Intelligent error recovery with positive-first messaging
- **Rich Diagnostics**: Integration with `miette` for beautiful error reports
- **Thread Safe**: Pure functional parsing with no global state
- **Production Ready**: 500+ tests covering all language features

---

## Quick Start

### Basic Usage

```rust
use ferra_parser::{Parser, Arena};
use ferra_lexer::Lexer;

// Parse a simple Ferra program
fn parse_ferra_code() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        fn calculate(x: i32, y: i32) -> i32 {
            return x + y * 2;
        }
    "#;
    
    // Create arena for AST allocation
    let arena = Arena::new();
    
    // Tokenize the source
    let tokens = Lexer::new(source).tokenize()?;
    
    // Parse into AST
    let mut parser = Parser::new(&arena, tokens);
    let ast = parser.parse_compilation_unit()?;
    
    println!("Parsed {} top-level items", ast.items.len());
    Ok(())
}
```

### Error Handling

```rust
use ferra_parser::{Parser, Arena, ParseError};
use ferra_lexer::Lexer;

fn parse_with_error_handling(source: &str) {
    let arena = Arena::new();
    let tokens = Lexer::new(source).tokenize().unwrap();
    let mut parser = Parser::new(&arena, tokens);
    
    match parser.parse_compilation_unit() {
        Ok(ast) => {
            println!("Successfully parsed {} items", ast.items.len());
        }
        Err(errors) => {
            for error in errors {
                eprintln!("Parse error: {}", error);
                // Rich diagnostic formatting with miette
                eprintln!("{:?}", error);
            }
        }
    }
}
```

---

## Core API Reference

### Parser Struct

The main entry point for parsing operations.

```rust
pub struct Parser<'arena, T: TokenStream> {
    // Private fields
}

impl<'arena, T: TokenStream> Parser<'arena, T> {
    /// Create a new parser with arena and token stream
    pub fn new(arena: &'arena Arena, tokens: T) -> Self;
    
    /// Parse a complete compilation unit (top-level program)
    pub fn parse_compilation_unit(&mut self) -> Result<&'arena CompilationUnit, Vec<ParseError>>;
    
    /// Parse a single expression
    pub fn parse_expression(&mut self) -> Result<&'arena Expression, ParseError>;
    
    /// Parse a single statement
    pub fn parse_statement(&mut self) -> Result<&'arena Statement, ParseError>;
    
    /// Parse a single top-level item (function, data class, etc.)
    pub fn parse_top_level_item(&mut self) -> Result<&'arena TopLevelItem, ParseError>;
}
```

### Arena Memory Management

Arena provides zero-copy AST allocation with automatic cleanup.

```rust
use ferra_parser::Arena;

// Create arena for AST nodes
let arena = Arena::new();

// All AST nodes are allocated in the arena
let ast = parser.parse_compilation_unit()?;

// Arena automatically deallocates when dropped
// No manual memory management required
```

### Token Stream Interface

The parser accepts any type implementing `TokenStream`:

```rust
pub trait TokenStream {
    /// Peek at the current token without consuming it
    fn peek(&self) -> &Token;
    
    /// Consume and return the current token
    fn consume(&mut self) -> Token;
    
    /// Check if we're at the end of the token stream
    fn is_at_end(&self) -> bool;
}

// Built-in implementation for Vec<Token>
let tokens: Vec<Token> = lexer.tokenize()?;
let parser = Parser::new(&arena, VecTokenStream::new(tokens));
```

---

## AST Node Reference

### Compilation Unit

The root AST node representing a complete Ferra source file.

```rust
pub struct CompilationUnit {
    pub items: Vec<&'arena TopLevelItem>,
    pub span: Span,
}

// Usage example
let ast = parser.parse_compilation_unit()?;
for item in &ast.items {
    match item {
        TopLevelItem::Function(func) => println!("Function: {}", func.name),
        TopLevelItem::DataClass(class) => println!("Data class: {}", class.name),
        TopLevelItem::VariableDecl(var) => println!("Variable: {}", var.name),
    }
}
```

### Expressions

All expression types with proper precedence handling:

```rust
pub enum Expression {
    // Literals
    StringLiteral(StringLiteral),
    IntegerLiteral(IntegerLiteral),
    FloatLiteral(FloatLiteral),
    BooleanLiteral(BooleanLiteral),
    
    // Complex expressions
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    FunctionCall(FunctionCall),
    MemberAccess(MemberAccess),
    ArrayIndex(ArrayIndex),
    ArrayLiteral(ArrayLiteral),
    
    // Control flow
    IfExpression(IfExpression),
    MatchExpression(MatchExpression),
    
    // Variables
    Identifier(Identifier),
    QualifiedIdentifier(QualifiedIdentifier),
}

// Expression parsing examples
let expr = parser.parse_expression()?;
match expr {
    Expression::Binary(bin) => {
        println!("Binary: {} {} {}", bin.left, bin.operator, bin.right);
    }
    Expression::FunctionCall(call) => {
        println!("Call: {}({})", call.function, call.arguments.len());
    }
    _ => {}
}
```

### Statements

All statement types including control flow:

```rust
pub enum Statement {
    // Declarations
    VariableDecl(VariableDecl),
    FunctionDecl(FunctionDecl),
    DataClassDecl(DataClassDecl),
    
    // Control flow
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
    ForStatement(ForStatement),
    ReturnStatement(ReturnStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    
    // Other
    ExpressionStatement(ExpressionStatement),
    BlockStatement(BlockStatement),
}
```

### Type System

Complete type annotation support:

```rust
pub enum Type {
    Identifier(String),           // i32, String, CustomType
    Tuple(Vec<&'arena Type>),     // (i32, String)
    Array(ArrayType),             // [i32; 10]
    Function(FunctionType),       // fn(i32, i32) -> i32
    Pointer(PointerType),         // *i32, *mut i32
    Generic(GenericType),         // Vec<T>, HashMap<K, V>
}

// Type parsing
let type_annotation = parser.parse_type()?;
match type_annotation {
    Type::Array(arr) => println!("Array of {} with size {}", arr.element_type, arr.size),
    Type::Function(func) => println!("Function with {} parameters", func.parameters.len()),
    _ => {}
}
```

---

## Advanced Usage

### Custom Token Streams

Implement custom token streams for specialized use cases:

```rust
use ferra_parser::TokenStream;

struct LazyTokenStream {
    source: String,
    position: usize,
}

impl TokenStream for LazyTokenStream {
    fn peek(&self) -> &Token {
        // Lazy tokenization
        self.tokenize_at_position(self.position)
    }
    
    fn consume(&mut self) -> Token {
        let token = self.peek().clone();
        self.position += 1;
        token
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.source_length()
    }
}
```

### Error Recovery Configuration

Configure error recovery behavior:

```rust
use ferra_parser::{Parser, ErrorCollector};

let arena = Arena::new();
let tokens = tokenize_source(source)?;
let mut parser = Parser::new(&arena, tokens);

// Configure error recovery
parser.set_max_errors(10);  // Stop after 10 errors
parser.set_recovery_mode(RecoveryMode::Aggressive);  // Try harder to recover

let result = parser.parse_compilation_unit();
```

### AST Visitor Pattern

Traverse AST nodes with the visitor pattern:

```rust
use ferra_parser::{AstVisitor, VisitResult};

struct AnalysisVisitor {
    function_count: usize,
    variable_count: usize,
}

impl AstVisitor for AnalysisVisitor {
    fn visit_function(&mut self, func: &FunctionDecl) -> VisitResult {
        self.function_count += 1;
        VisitResult::Continue
    }
    
    fn visit_variable(&mut self, var: &VariableDecl) -> VisitResult {
        self.variable_count += 1;
        VisitResult::Continue
    }
}

// Usage
let mut visitor = AnalysisVisitor::new();
ast.accept(&mut visitor);
println!("Found {} functions and {} variables", 
         visitor.function_count, visitor.variable_count);
```

### Performance Optimization

Best practices for high-performance parsing:

```rust
// Reuse arena for multiple parse operations
let arena = Arena::with_capacity(1024 * 1024);  // 1MB initial capacity

// Parse multiple files efficiently
for source_file in source_files {
    let tokens = tokenize(source_file)?;
    let mut parser = Parser::new(&arena, tokens);
    let ast = parser.parse_compilation_unit()?;
    
    // Process AST...
    
    // Arena automatically resets after drop
}

// For very large files, consider streaming parsing
let mut streaming_parser = StreamingParser::new(&arena);
for chunk in source_chunks {
    streaming_parser.parse_chunk(chunk)?;
}
let complete_ast = streaming_parser.finalize()?;
```

---

## Integration Examples

### IDE Integration

```rust
use ferra_parser::{Parser, Arena, CompilationUnit};

pub struct LanguageServer {
    arena: Arena,
}

impl LanguageServer {
    pub fn parse_document(&self, uri: &str, content: &str) -> ParseResult {
        let tokens = ferra_lexer::tokenize(content)?;
        let mut parser = Parser::new(&self.arena, tokens);
        
        match parser.parse_compilation_unit() {
            Ok(ast) => ParseResult::Success(ast),
            Err(errors) => ParseResult::Errors(errors),
        }
    }
    
    pub fn get_diagnostics(&self, ast: &CompilationUnit) -> Vec<Diagnostic> {
        // Extract diagnostics from AST
        DiagnosticExtractor::new().extract_diagnostics(ast)
    }
}
```

### REPL Integration

```rust
use ferra_parser::{Parser, Arena, Expression};

pub struct ReplParser {
    arena: Arena,
}

impl ReplParser {
    pub fn parse_expression(&self, input: &str) -> Result<&Expression, ParseError> {
        let tokens = ferra_lexer::tokenize(input)?;
        let mut parser = Parser::new(&self.arena, tokens);
        parser.parse_expression()
    }
    
    pub fn parse_statement(&self, input: &str) -> Result<Statement, ParseError> {
        let tokens = ferra_lexer::tokenize(input)?;
        let mut parser = Parser::new(&self.arena, tokens);
        parser.parse_statement()
    }
}

// REPL usage
let repl = ReplParser::new();
match repl.parse_expression("2 + 3 * 4") {
    Ok(expr) => println!("Expression: {:?}", expr),
    Err(err) => println!("Error: {}", err),
}
```

### Build Tool Integration

```rust
use ferra_parser::{Parser, Arena};
use std::fs;

pub fn compile_project(project_path: &Path) -> Result<CompilationResult, CompileError> {
    let arena = Arena::new();
    let mut compilation_units = Vec::new();
    
    // Parse all .ferra files in project
    for entry in fs::read_dir(project_path)? {
        let path = entry?.path();
        if path.extension() == Some("ferra".as_ref()) {
            let content = fs::read_to_string(&path)?;
            let tokens = ferra_lexer::tokenize(&content)?;
            let mut parser = Parser::new(&arena, tokens);
            
            match parser.parse_compilation_unit() {
                Ok(ast) => compilation_units.push(ast),
                Err(errors) => return Err(CompileError::ParseErrors(errors)),
            }
        }
    }
    
    Ok(CompilationResult { units: compilation_units })
}
```

---

## Error Handling Guide

### Error Types

The parser produces structured errors with rich context:

```rust
pub enum ParseError {
    UnexpectedToken { expected: String, found: Token, span: Span },
    UnexpectedEof { expected: String, span: Span },
    InvalidExpression { message: String, span: Span },
    InvalidStatement { message: String, span: Span },
    TooManyErrors { count: usize },
}

impl ParseError {
    /// Get human-readable error message
    pub fn message(&self) -> String;
    
    /// Get source location
    pub fn span(&self) -> Span;
    
    /// Get suggested fixes
    pub fn suggestions(&self) -> Vec<String>;
}
```

### Error Recovery

The parser includes intelligent error recovery:

```rust
// Parser automatically recovers from most syntax errors
let source = r#"
    fn broken_function( {  // Missing parameters
        let x = 5;
        return x;
    }
    
    fn working_function() -> i32 {  // This still parses correctly
        return 42;
    }
"#;

let tokens = tokenize(source)?;
let mut parser = Parser::new(&arena, tokens);

// Returns partial AST with error list
match parser.parse_compilation_unit() {
    Ok(ast) => unreachable!(),  // Won't happen with syntax errors
    Err(errors) => {
        // AST may still contain successfully parsed items
        let partial_ast = parser.partial_result();
        println!("Parsed {} items despite {} errors", 
                 partial_ast.items.len(), errors.len());
    }
}
```

### Diagnostic Integration

Rich error reporting with `miette`:

```rust
use miette::{Result, WrapErr};

fn parse_with_diagnostics(source: &str) -> Result<CompilationUnit> {
    let tokens = ferra_lexer::tokenize(source)
        .wrap_err("Failed to tokenize source")?;
    
    let arena = Arena::new();
    let mut parser = Parser::new(&arena, tokens);
    
    parser.parse_compilation_unit()
        .map_err(|errors| {
            // Convert ParseError to miette::Error for rich formatting
            miette::Error::from(errors)
        })
        .wrap_err("Failed to parse Ferra source code")
}
```

---

## Performance Guide

### Memory Management

The arena allocator provides optimal memory usage:

```rust
// Arena statistics
let arena = Arena::new();
println!("Initial capacity: {} bytes", arena.capacity());

// Parse large program
let ast = parser.parse_compilation_unit()?;

println!("Used memory: {} bytes", arena.used());
println!("Allocated nodes: {}", arena.node_count());

// Arena automatically releases all memory when dropped
drop(arena);  // All AST nodes become invalid
```

### Parsing Performance

Best practices for optimal parsing performance:

```rust
// 1. Pre-allocate arena for known workloads
let arena = Arena::with_capacity(estimated_size);

// 2. Reuse arena for multiple small parses
for small_program in small_programs {
    let ast = parse_in_arena(&arena, small_program)?;
    process_ast(ast);
    arena.reset();  // Clear for next iteration
}

// 3. Use streaming for very large files
let streaming_parser = StreamingParser::new(&arena);
for chunk in file_chunks {
    streaming_parser.add_chunk(chunk);
}
let ast = streaming_parser.finalize()?;

// 4. Configure error recovery for performance
parser.set_max_errors(5);  // Stop early on heavily broken code
parser.set_recovery_mode(RecoveryMode::Fast);  // Prefer speed over thoroughness
```

### Benchmarking

Use the built-in benchmarks to measure performance:

```bash
# Run parser benchmarks
cargo bench --package ferra_parser

# Specific benchmark categories
cargo bench parser_creation
cargo bench expression_parsing
cargo bench statement_parsing
cargo bench memory_profiling
```

---

## Migration Guide

### From Parser v0.x to v1.0

Major changes in the v1.0 API:

```rust
// OLD (v0.x)
let parser = Parser::new(source);
let ast = parser.parse()?;

// NEW (v1.0)
let arena = Arena::new();
let tokens = ferra_lexer::tokenize(source)?;
let mut parser = Parser::new(&arena, tokens);
let ast = parser.parse_compilation_unit()?;
```

### Breaking Changes

1. **Arena Requirement**: All AST nodes now require an arena
2. **Token Stream**: Parser now takes pre-tokenized input
3. **Error Types**: Comprehensive error type hierarchy
4. **Memory Model**: Zero-copy AST with lifetime management

### Migration Steps

1. Add arena allocation: `let arena = Arena::new();`
2. Pre-tokenize input: `let tokens = ferra_lexer::tokenize(source)?;`
3. Update error handling: `match parser.parse_compilation_unit()`
4. Update AST node access patterns for arena lifetimes

---

## Contributing

See `CONTRIBUTING.md` for development guidelines, testing procedures, and contribution workflows.

### Running Tests

```bash
# Run all parser tests
cargo test --package ferra_parser

# Run specific test categories
cargo test test_expressions
cargo test test_control_flow
cargo test test_error_recovery

# Run with coverage
cargo tarpaulin --package ferra_parser
```

### Documentation

```bash
# Generate API documentation
cargo doc --package ferra_parser --open

# Check documentation coverage
cargo doc --package ferra_parser -- -D missing_docs
``` 